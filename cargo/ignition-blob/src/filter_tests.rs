use crate::file_system::in_memory::InMemoryFileSystem;

#[tokio::test]
async fn get_success() {
    let file_system = InMemoryFileSystem::new();
    file_system.write(
        "data/sha256/31/5f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3",
        b"Hello, world!".to_vec(),
    );
    let filter = super::filter(file_system);
    let resp = warp::test::request()
        .path("/blobs/sha256:315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.body(), "Hello, world!");
}

#[tokio::test]
async fn get_not_found() {
    let file_system = InMemoryFileSystem::new();
    let filter = super::filter(file_system);
    let resp = warp::test::request()
        .path("/blobs/sha256:315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn put_success() {
    let file_system = InMemoryFileSystem::new();
    let filter = super::filter(file_system.clone());
    let resp = warp::test::request()
        .method("POST")
        .path("/blobs")
        .body("Hello, world!")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.body(),
        "sha256:315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
    );
    assert_eq!(
        file_system
            .read("data/sha256/31/5f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3")
            .as_ref()
            .map(|arc_box_content| -> &[u8] { &*arc_box_content }),
        Some("Hello, world!".as_bytes()),
    );
}
