use crate::file_system::{tokio::tokio_file_system, FileSystem};
use bytes::buf::BufExt;
use sha2::{Digest, Sha256};
use std::io::{self, Read, SeekFrom};
use std::path::{Path, PathBuf};
use tokio::prelude::*;
use warp::Filter;

mod file_system;

#[cfg(test)]
mod filter_tests;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    warp::serve(filter(tokio_file_system()))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn filter<F>(file_system: F) -> impl Filter<Extract = impl warp::Reply> + Clone
where
    F: FileSystem + 'static,
{
    let specific_blob = warp::path!(BlobId)
        .and(warp::get())
        .and(with_file_system::<F>(file_system.clone()))
        .and_then(get_blob::<F>);
    let blob_collection = warp::path::end()
        .and(warp::post())
        .and(warp::filters::body::aggregate())
        .and(with_file_system::<F>(file_system.clone()))
        .and_then(post_blob::<_, F>);

    warp::path("blobs").and(specific_blob.or(blob_collection))
}

fn with_file_system<F>(
    file_system: F,
) -> impl Filter<Extract = (F,), Error = std::convert::Infallible> + Clone
where
    F: FileSystem + 'static,
{
    warp::any().map(move || file_system.clone())
}

async fn get_blob<F>(id: BlobId, file_system: F) -> Result<impl warp::Reply, warp::Rejection>
where
    F: FileSystem + 'static,
{
    let path = path_for_id(&id);
    let fut_file = file_system.open(AsRef::<Path>::as_ref(&path));
    let mut file = match fut_file.await {
        Ok(file) => file,
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => return Err(warp::reject::not_found()),
            _ => return Err(internal_server_error()),
        },
    };

    // ASSUMPTION: The file will not change size between observing the
    // length and reading its content. This is reasonable if no other
    // process is interfering with this server's data directory.
    let len = file
        .seek(SeekFrom::End(0))
        .await
        .map_err(|_| internal_server_error())?;
    file.seek(SeekFrom::Start(0))
        .await
        .map_err(|_| internal_server_error())?;

    let (mut sender, body) = hyper::Body::channel();
    tokio::spawn(async move {
        // TODO: tune buffer size
        let mut buf = [0; 4096];
        loop {
            let n = file.read(&mut buf[..]).await.unwrap();
            if n == 0 {
                break;
            }
            sender
                .send_data(hyper::body::Bytes::copy_from_slice(&buf[..n]))
                .await
                .unwrap();
        }
    });
    Ok(http::Response::builder()
        .status(200)
        .header("Content-Length", len)
        .body(body)
        .unwrap())
}

async fn post_blob<B, F>(body: B, file_system: F) -> Result<impl warp::Reply, warp::Rejection>
where
    B: bytes::Buf,
    F: FileSystem + 'static,
{
    // Allocate a temporary file and get its path.
    let temp_path = {
        let fut = file_system.make_temporary_file();
        fut.await.map_err(|e| {
            log::error!("unexpected error making temporary file: {}", e);
            internal_server_error()
        })?
    };
    // Open the temporary file for use as a streaming destination.
    let mut temp_file = {
        let fut = file_system.create(&temp_path);
        fut.await.map_err(|e| {
            log::error!(
                "unexpected error opening temporary file {:?}: {}",
                temp_path,
                e,
            );
            internal_server_error()
        })?
    };

    // Stream the request body both to a hasher and to the temporary
    // file.
    let mut body = body.reader();
    let mut hasher = Sha256::new();
    // TODO: tune buffer size
    let mut buf = [0; 4096];
    loop {
        let n = body.read(&mut buf[..]).map_err(|e| {
            log::error!("unexpected error reading request body: {}", e);
            internal_server_error()
        })?;
        if n == 0 {
            // Successful EOF.
            break;
        }
        let buf = &buf[..n];
        hasher.update(buf);
        let fut = temp_file.write_all(&buf);
        fut.await.map_err(|e| {
            log::error!("unexpected error writing temporary file: {}", e);
            internal_server_error()
        })?;
    }

    // Finalize the hash and construct the destination path.
    let id = BlobId {
        algorithm: HashAlgorithm::Sha256,
        hash: hasher.finalize().to_vec().into_boxed_slice(),
    };
    let dest_path = path_for_id(&id);

    // Flush and close the temporary file. A successful call to
    // flush() ensures the file will be closed immediately when it's
    // dropped.
    let fut = temp_file.flush();
    fut.await.map_err(|e| {
        log::error!("unexpected error flushing temporary file: {}", e);
        internal_server_error()
    })?;
    drop(temp_file);

    // Rename the temporary file into its destination.
    let parent_dir = dest_path.parent().unwrap();
    let fut = file_system.create_dir_all(parent_dir);
    fut.await.map_err(|e| {
        log::error!(
            "unexpected error creating destination directory {:?}: {}",
            parent_dir,
            e,
        );
        internal_server_error()
    })?;
    let fut = file_system.rename(&temp_path, &dest_path);
    fut.await.map_err(|e| {
        log::error!(
            "unexpected error renaming temporary file from {:?} to {:?}: {}",
            temp_path,
            dest_path,
            e,
        );
        internal_server_error()
    })?;

    Ok(http::Response::builder()
        .status(200)
        .body(id.to_string())
        .unwrap())
}

#[derive(Debug)]
struct InternalServerError;
impl warp::reject::Reject for InternalServerError {}
fn internal_server_error() -> warp::reject::Rejection {
    warp::reject::custom(InternalServerError)
}

#[derive(Debug, Eq, PartialEq)]
struct BlobId {
    algorithm: HashAlgorithm,
    hash: Box<[u8]>,
}
impl std::str::FromStr for BlobId {
    type Err = ();
    fn from_str(s: &str) -> Result<BlobId, ()> {
        let mut parts = s.splitn(2, ':');
        let algorithm = HashAlgorithm::parse(parts.next().ok_or(())?).ok_or(())?;
        let mut hash = Box::new([0; 32]);
        hex::decode_to_slice(parts.next().ok_or(())?, &mut *hash).map_err(|_| ())?;
        Ok(BlobId { algorithm, hash })
    }
}
impl std::fmt::Display for BlobId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.algorithm)?;
        let mut hash_hex = [0; 64];
        hex::encode_to_slice(&*self.hash, &mut hash_hex[..]).unwrap();
        let hash_hex = std::str::from_utf8(&hash_hex[..]).unwrap();
        write!(f, "{}", hash_hex)
    }
}

#[cfg(test)]
mod blob_id_tests {
    use super::{BlobId, HashAlgorithm};

    #[test]
    fn from_str_success() {
        assert_eq!(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".parse(),
            Ok(BlobId {
                algorithm: HashAlgorithm::Sha256,
                hash: vec![
                    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89,
                    0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23,
                    0x45, 0x67, 0x89, 0xab, 0xcd, 0xef
                ]
                .into_boxed_slice(),
            }),
        );
    }

    #[test]
    fn from_str_bad_algorithm() {
        assert_eq!(
            "md5:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .parse::<BlobId>(),
            Err(()),
        );
    }

    #[test]
    fn from_str_hash_too_long() {
        assert_eq!(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0"
                .parse::<BlobId>(),
            Err(()),
        );
    }

    #[test]
    fn from_str_hash_too_short() {
        assert_eq!(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde"
                .parse::<BlobId>(),
            Err(()),
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HashAlgorithm {
    Sha256,
}
impl HashAlgorithm {
    fn parse(text: &str) -> Option<HashAlgorithm> {
        match text {
            "sha256" => Some(HashAlgorithm::Sha256),
            _ => None,
        }
    }
}
impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:",
            match self {
                HashAlgorithm::Sha256 => "sha256",
            }
        )
    }
}

#[cfg(test)]
mod hash_algorithm_tests {
    use super::HashAlgorithm;

    #[test]
    fn parse() {
        assert_eq!(HashAlgorithm::parse("sha256"), Some(HashAlgorithm::Sha256));
    }
}

fn path_for_id(id: &BlobId) -> PathBuf {
    match id.algorithm {
        HashAlgorithm::Sha256 => {
            assert_eq!(id.hash.len(), 32);
            let mut hash_hex = [0; 64];
            hex::encode_to_slice(&id.hash[..], &mut hash_hex[..]).unwrap();
            let hash_hex_str = std::str::from_utf8(&hash_hex[..]).unwrap();
            let mut path = PathBuf::from("data/sha256");
            path.push(&hash_hex_str[..2]);
            path.push(&hash_hex_str[2..]);
            path
        }
    }
}
