#![cfg(unix)]

mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn eviction_while_binding_never_resurrects_a_stale_generation() {
    let seed = store_fixture::spaced_store(
        "generation-eviction-race",
        store_fixture::limits(16, 0),
        1_000,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    let generation =
        SourceGeneration::try_new("https://cdn.example/video", "\"generation-one\"", 8).unwrap();
    seed.store
        .bind_representation(binding.clone())
        .await
        .unwrap();
    seed.store.select_transfer(transfer.clone()).await.unwrap();
    seed.store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();
    assert!(seed
        .store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"data")
        .await
        .unwrap());

    let reopened = store_fixture::reopened(&seed);
    reopened.store.load_existing().await.unwrap();
    let root = reopened.root.clone();
    let store = Arc::new(reopened.store);
    let generation_path = root.join("clip.generation.json");
    let payload = tokio::fs::read(&generation_path).await.unwrap();
    tokio::fs::remove_file(&generation_path).await.unwrap();
    make_fifo(&generation_path);
    let (written, observed) = oneshot::channel();
    let (close, release) = oneshot::channel();
    let writer = tokio::spawn(async move {
        let mut fifo = tokio::fs::OpenOptions::new()
            .write(true)
            .open(generation_path)
            .await
            .unwrap();
        fifo.write_all(&payload).await.unwrap();
        written.send(()).unwrap();
        release.await.unwrap();
    });
    let binding_store = store.clone();
    let bind = tokio::spawn(async move { binding_store.bind_representation(binding).await });
    tokio::time::timeout(Duration::from_secs(2), observed)
        .await
        .unwrap()
        .unwrap();

    store.set_storage_budget(1).await.unwrap();
    close.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), bind)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    writer.await.unwrap();
    assert_eq!(store.used_bytes().await, 0);
    assert!(!root.join("clip.ranges.json").exists());
    assert_eq!(store.select_transfer(transfer).await.unwrap(), None);
    store_fixture::discard(&root);
}

fn make_fifo(path: &std::path::Path) {
    let path = CString::new(path.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
