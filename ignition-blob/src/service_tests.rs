use futures_util::stream::{iter, TryStreamExt};
use ignition_blob_proto::blob_pb;
use ignition_blob_proto::blob_pb::blob_service_client::BlobServiceClient;
use ignition_blob_proto::blob_pb::blob_service_server::BlobServiceServer;
use testable_file_system::InMemoryFileSystem;

use crate::BlobServiceImpl;

fn make_client(
    file_system: &InMemoryFileSystem,
) -> BlobServiceClient<BlobServiceServer<BlobServiceImpl<InMemoryFileSystem>>> {
    BlobServiceClient::new(BlobServiceServer::new(BlobServiceImpl {
        file_system: file_system.clone(),
    }))
}

#[tokio::test]
async fn get_success() {
    let file_system = InMemoryFileSystem::new();
    file_system.write(
        "data/blake3/ed/e5c0b10f2ec4979c69b52f61e42ff5b413519ce09be0f14d098dcfe5f6f98d",
        b"Hello, world!".to_vec(),
    );
    let mut client = make_client(&file_system);
    assert_eq!(
        client
            .get(blob_pb::GetRequest {
                id: Some(blob_pb::BlobId {
                    algorithm: blob_pb::HashAlgorithm::Blake3 as i32,
                    hash: hex::decode(
                        "ede5c0b10f2ec4979c69b52f61e42ff5b413519ce09be0f14d098dcfe5f6f98d"
                    )
                    .unwrap(),
                    ..Default::default()
                }),
                max_chunk_size: 8,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner()
            .try_collect::<Vec<_>>()
            .await
            .unwrap(),
        vec![
            blob_pb::GetResponse {
                total_byte_length: 13,
                ..Default::default()
            },
            blob_pb::GetResponse {
                data: b"Hello, w".to_vec(),
                ..Default::default()
            },
            blob_pb::GetResponse {
                data: b"orld!".to_vec(),
                ..Default::default()
            },
        ],
    );
}

#[tokio::test]
async fn get_not_found() {
    let file_system = InMemoryFileSystem::new();
    let mut client = make_client(&file_system);
    assert_eq!(
        client
            .get(blob_pb::GetRequest {
                id: Some(blob_pb::BlobId {
                    algorithm: blob_pb::HashAlgorithm::Blake3 as i32,
                    hash: hex::decode(
                        "ede5c0b10f2ec4979c69b52f61e42ff5b413519ce09be0f14d098dcfe5f6f98d"
                    )
                    .unwrap(),
                    ..Default::default()
                }),
                max_chunk_size: 8,
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound,
    );
}

#[tokio::test]
async fn put_success() {
    let file_system = InMemoryFileSystem::new();
    let mut client = make_client(&file_system);
    assert_eq!(
        client
            .put(iter(vec![
                blob_pb::PutRequest {
                    data: b"Hello".to_vec(),
                    ..Default::default()
                },
                blob_pb::PutRequest {
                    data: b", world!".to_vec(),
                    ..Default::default()
                },
            ]))
            .await
            .unwrap()
            .into_inner(),
        blob_pb::PutResponse {
            id: Some(blob_pb::BlobId {
                algorithm: blob_pb::HashAlgorithm::Blake3 as i32,
                hash: hex::decode(
                    "ede5c0b10f2ec4979c69b52f61e42ff5b413519ce09be0f14d098dcfe5f6f98d"
                )
                .unwrap(),
                ..Default::default()
            }),
            ..Default::default()
        },
    );
    assert_eq!(
        file_system
            .read("data/blake3/ed/e5c0b10f2ec4979c69b52f61e42ff5b413519ce09be0f14d098dcfe5f6f98d")
            .as_ref()
            .map(|arc_box_content| -> &[u8] { &*arc_box_content }),
        Some("Hello, world!".as_bytes()),
    );
}
