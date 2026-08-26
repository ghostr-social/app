use crate::partial_range_store::RepresentationRead;
use core::time::Duration;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use sha2::{Digest as _, Sha256};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn playback_read_is_not_blocked_by_an_unrelated_action_write() {
    let mut fixture = crate::tests::paused_fixture::paused_store_after("parallel-read-write", 1);
    seed_ready_video(&fixture.root);
    let (binding, identity) = loading_identity();
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    let action = fixture
        .store
        .reserve_action(&identity, 1, 4)
        .await
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(identity.source().as_str(), "\"g\"", 4)
        .expect("valid test fixture");
    fixture
        .store
        .open_sparse_response(&identity, &action, generation.clone(), ByteRange::new(0, 4))
        .await
        .expect("valid test fixture");
    let (ready_binding, revision) = fixture.store.stream_snapshot("ready").await;
    let writer = spawn_write(&fixture.store, identity, generation, action.clone());
    fixture.wait_until_admission().await;

    let read = tokio::time::timeout(
        Duration::from_millis(250),
        fixture
            .store
            .read_for_stream("ready", ready_binding.as_ref(), revision, 0..8),
    )
    .await;
    fixture.resume();
    assert!(writer
        .await
        .expect("valid test fixture")
        .expect("valid test fixture"));
    fixture.store.release_action(&action).await;
    assert_present(
        read.expect("unrelated write blocked playback")
            .expect("valid test fixture"),
    );
    crate::tests::store_fixture::discard(&fixture.root);
}

fn spawn_write(
    store: &std::sync::Arc<crate::partial_range_store::PartialRangeStore>,
    identity: ghostr_engine::representation::TransferIdentity,
    generation: SourceGeneration,
    action: crate::partial_range_store::StoreAction,
) -> tokio::task::JoinHandle<anyhow::Result<bool>> {
    let store = std::sync::Arc::clone(store);
    tokio::spawn(async move {
        store
            .write_range_for_action_if_current(&identity, &generation, &action, 0, b"load")
            .await
    })
}

fn loading_identity() -> (
    ghostr_engine::representation::RepresentationBinding,
    ghostr_engine::representation::TransferIdentity,
) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("loading"), meta());
    let identity = binding
        .transfer("https://cdn.example/loading")
        .expect("valid test fixture");
    (binding, identity)
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/loading".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(4),
        duration_ms: Some(1_000),
    }
}

fn seed_ready_video(root: &std::path::Path) {
    let bytes = b"abcdefgh";
    std::fs::create_dir_all(root).expect("valid test fixture");
    std::fs::write(root.join("ready.part"), bytes).expect("valid test fixture");
    let sha256 = format!("{:x}", Sha256::digest(bytes));
    let manifest = format!(
        r#"{{"version":2,"total_len":8,"intervals":[{{"start":0,"end":8,"sha256":"{sha256}"}}]}}"#
    );
    std::fs::write(root.join("ready.ranges.json"), manifest).expect("valid test fixture");
}

fn assert_present(read: RepresentationRead) {
    assert!(
        matches!(read, RepresentationRead::Present(bytes) if bytes == b"abcdefgh"),
        "parallel read should return the complete stored bytes"
    );
}
