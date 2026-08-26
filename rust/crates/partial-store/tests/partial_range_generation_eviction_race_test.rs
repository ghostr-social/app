#![cfg(unix)]

use core::time::Duration;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use nix::sys::stat::Mode;
use std::sync::Arc;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::oneshot;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn eviction_while_binding_never_resurrects_a_stale_generation() {
    let seed = crate::tests::store_fixture::spaced_store(
        "generation-eviction-race",
        crate::tests::store_fixture::limits(16, 0),
        1_000,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let transfer = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    let generation =
        SourceGeneration::try_new("https://cdn.example/video", "\"generation-one\"", 8)
            .expect("valid test fixture");
    seed.store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    seed.store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");
    seed.store
        .accept_generation(&transfer, generation.clone())
        .await
        .expect("valid test fixture");
    assert!(seed
        .store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"data")
        .await
        .expect("valid test fixture"));

    let reopened = crate::tests::store_fixture::reopened(&seed);
    reopened
        .store
        .load_existing()
        .await
        .expect("valid test fixture");
    let root = reopened.root.clone();
    let store = Arc::new(reopened.store);
    let generation_path = root.join("clip.generation.json");
    let payload = tokio::fs::read(&generation_path)
        .await
        .expect("valid test fixture");
    tokio::fs::remove_file(&generation_path)
        .await
        .expect("valid test fixture");
    make_fifo(&generation_path);
    let (written, observed) = oneshot::channel();
    let (close, release) = oneshot::channel();
    let writer = tokio::spawn(async move {
        let mut fifo = tokio::fs::OpenOptions::new()
            .write(true)
            .open(generation_path)
            .await
            .expect("valid test fixture");
        fifo.write_all(&payload).await.expect("valid test fixture");
        written.send(()).expect("valid test fixture");
        release.await.expect("valid test fixture");
    });
    let binding_store = std::sync::Arc::clone(&store);
    let bind = tokio::spawn(async move { binding_store.bind_representation(binding).await });
    tokio::time::timeout(Duration::from_secs(2), observed)
        .await
        .expect("valid test fixture")
        .expect("valid test fixture");

    store
        .set_storage_budget(1)
        .await
        .expect("valid test fixture");
    close.send(()).expect("valid test fixture");
    tokio::time::timeout(Duration::from_secs(2), bind)
        .await
        .expect("valid test fixture")
        .expect("valid test fixture")
        .expect("valid test fixture");
    writer.await.expect("valid test fixture");
    assert_eq!(store.used_bytes().await, 0);
    assert!(!root.join("clip.ranges.json").exists());
    assert_eq!(
        store
            .select_transfer(transfer)
            .await
            .expect("valid test fixture"),
        None
    );
    crate::tests::store_fixture::discard(&root);
}

fn make_fifo(path: &std::path::Path) {
    let owner_access = Mode::S_IRUSR | Mode::S_IWUSR;
    nix::unistd::mkfifo(path, owner_access).expect("test FIFO should be created");
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
