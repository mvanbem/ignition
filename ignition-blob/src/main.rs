use ignition_blob_proto::blob_pb;
use std::convert::TryInto;
use std::io::{self, SeekFrom};
use std::path::PathBuf;
use testable_file_system::{real_file_system, FileSystem};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

#[cfg(test)]
mod service_tests;

const DEFAULT_MAX_CHUNK_SIZE: usize = 1048576;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let addr = "[::1]:3030".parse()?;

    tonic::transport::Server::builder()
        .add_service(blob_pb::blob_service_server::BlobServiceServer::new(
            BlobServiceImpl {
                file_system: real_file_system(),
            },
        ))
        .serve(addr)
        .await?;

    Ok(())
}

fn unexpected_io_error(e: impl std::error::Error) -> Status {
    log::error!("unexpected I/O error: {}", e);
    Status::internal(&format!("unexpected I/O error: {}", e))
}

struct BlobServiceImpl<F: FileSystem> {
    file_system: F,
}

#[tonic::async_trait]
impl<F: FileSystem + 'static> blob_pb::blob_service_server::BlobService for BlobServiceImpl<F> {
    type GetStream = ReceiverStream<Result<blob_pb::GetResponse, Status>>;

    async fn get(
        &self,
        request: Request<blob_pb::GetRequest>,
    ) -> Result<Response<Self::GetStream>, Status> {
        let id = request
            .get_ref()
            .id
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("id field was unset"))?;

        let path = path_for_id(&id).map_err(|PathForIdError::UnknownHashAlgorithm| {
            Status::invalid_argument("unknown hash algorithm")
        })?;

        let mut file = self
            .file_system
            .open(&path)
            .await
            .map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => Status::not_found("blob not found"),
                _ => unexpected_io_error(e),
            })?;

        // ASSUMPTION: The file will not change size between observing the
        // length and reading its content. This is reasonable if no other
        // process is interfering with this server's data directory.
        let len = file
            .seek(SeekFrom::End(0))
            .await
            .map_err(unexpected_io_error)?;
        file.seek(SeekFrom::Start(0))
            .await
            .map_err(unexpected_io_error)?;

        // Spawn a task to stream data to the client.
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            // The first response just indicates the overall length.
            tx.send(Ok(blob_pb::GetResponse {
                total_byte_length: len,
                data: vec![],
            }))
            .await
            .unwrap();

            // Send a response for each data chunk.
            let max_chunk_size = match request.get_ref().max_chunk_size {
                0 => DEFAULT_MAX_CHUNK_SIZE,
                x => x.try_into().unwrap(),
            };
            let mut buf = vec![0; max_chunk_size];
            loop {
                let n = file.read(&mut buf[..]).await.unwrap();
                if n == 0 {
                    break;
                }

                tx.send(Ok(blob_pb::GetResponse {
                    total_byte_length: 0,
                    data: buf[..n].to_vec(),
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn put(
        &self,
        mut request: Request<Streaming<blob_pb::PutRequest>>,
    ) -> Result<Response<blob_pb::PutResponse>, Status> {
        // Allocate a temporary file and get its path.
        let temp_path = self
            .file_system
            .make_temporary_file()
            .await
            .map_err(unexpected_io_error)?;

        // Open the temporary file for use as a streaming destination.
        let mut temp_file = self
            .file_system
            .create(&temp_path)
            .await
            .map_err(unexpected_io_error)?;

        // Stream the request body to both a hasher and the temporary file.
        let mut hasher = blake3::Hasher::new();
        while let Some(request) = request.get_mut().message().await? {
            hasher.update(&*request.data);
            temp_file
                .write_all(&*request.data)
                .await
                .map_err(unexpected_io_error)?;
        }

        // Finalize the hash and construct the destination path.
        let id = blob_pb::BlobId {
            algorithm: blob_pb::HashAlgorithm::Blake3 as i32,
            hash: hasher.finalize().as_bytes().to_vec(),
        };
        let dest_path = path_for_id(&id).unwrap();

        // Flush and close the temporary file. A successful call to flush() ensures the file will be
        // closed immediately when it's dropped.
        temp_file.flush().await.map_err(unexpected_io_error)?;
        drop(temp_file);

        // Rename the temporary file into its destination.
        let parent_dir = dest_path.parent().unwrap();
        self.file_system
            .create_dir_all(parent_dir)
            .await
            .map_err(unexpected_io_error)?;
        self.file_system
            .rename(&temp_path, &dest_path)
            .await
            .map_err(unexpected_io_error)?;

        Ok(Response::new(blob_pb::PutResponse { id: Some(id) }))
    }
}

#[derive(Debug, Error)]
enum PathForIdError {
    #[error("unknown hash algorithm")]
    UnknownHashAlgorithm,
}

fn path_for_id(id: &blob_pb::BlobId) -> Result<PathBuf, PathForIdError> {
    match id.algorithm() {
        blob_pb::HashAlgorithm::Unknown => Err(PathForIdError::UnknownHashAlgorithm),

        blob_pb::HashAlgorithm::Blake3 => {
            assert_eq!(id.hash.len(), 32);
            let mut hash_hex = [0; 64];
            hex::encode_to_slice(&id.hash[..], &mut hash_hex[..]).unwrap();
            let hash_hex_str = std::str::from_utf8(&hash_hex[..]).unwrap();
            let mut path = PathBuf::from("data/blake3");
            path.push(&hash_hex_str[..2]);
            path.push(&hash_hex_str[2..]);
            Ok(path)
        }
    }
}
