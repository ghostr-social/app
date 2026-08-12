mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::RepresentationRead;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn current_representation_reports_a_missing_unstored_span() {
    let root = store_fixture::temp_root("partial-representation-missing");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let binding = Catalog::new().upsert(PostId::new("clip"), meta());
    store.bind_representation(binding.clone()).await.unwrap();

    let read = store.read_for_representation(&binding, 0..4).await.unwrap();

    assert!(matches!(read, RepresentationRead::Missing));
    store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://video.example/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
