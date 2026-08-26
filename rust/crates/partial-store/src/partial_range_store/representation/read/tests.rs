use super::{ContentRevision, RepresentationRead};
use crate::partial_range_store::capacity::StoreCapacity;
use crate::partial_range_store::PartialRangeStore;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[tokio::test]
async fn an_error_from_a_replaced_stream_is_reported_as_superseded() {
    let (_root, store) = store();
    let revision = ContentRevision::default();
    store.advance_content_revision("clip").await;

    let read = Err(anyhow::anyhow!("stale read failure"));
    assert!(matches!(
        store
            .finish_stream_read("clip", None, revision, read)
            .await
            .expect("valid test fixture"),
        RepresentationRead::Superseded
    ));
}

#[tokio::test]
async fn a_stream_authority_check_preserves_store_failure() {
    let (root, store) = store();
    std::fs::create_dir_all(root.join("clip.transform.video")).expect("valid test fixture");

    assert!(store
        .stream_is_current("clip", None, ContentRevision::default())
        .await
        .is_err());
}

#[tokio::test]
async fn a_representation_authority_check_preserves_store_failure() {
    let (root, store) = store();
    let binding = Catalog::new().upsert(
        PostId::new("clip"),
        VideoMeta {
            urls: vec!["https://media.example/clip.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(2_000),
        },
    );
    std::fs::create_dir_all(root.join("clip.transform.video")).expect("valid test fixture");

    assert!(store.read_for_representation(&binding, 0..1).await.is_err());
}

fn store() -> (std::path::PathBuf, PartialRangeStore) {
    let root = std::env::temp_dir().join(format!(
        "ghostr-stale-read-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("valid test fixture")
            .as_nanos()
    ));
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    (root, store)
}
