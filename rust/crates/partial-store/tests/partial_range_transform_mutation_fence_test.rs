mod store_fixture;

use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::{TransformFence, TransformPublication};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn transformed_output_cannot_cross_an_input_representation_change() {
    let root = store_fixture::temp_root("partial-transform-fence");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let first = catalog.upsert(PostId::new("post"), meta("first"));
    store.bind_representation(first.clone()).await.unwrap();
    store.write_range("post", 0, b"input").await.unwrap();
    store.set_total_len("post", 5).await.unwrap();
    store.finalize("post", None).await.unwrap();
    let revision = store.media_snapshot("post").await.unwrap().revision();
    let next = catalog.upsert(PostId::new("post"), meta("next"));
    store.bind_representation(next.clone()).await.unwrap();
    let stale = TransformPublication::try_new(
        TransformFence::new(first, revision),
        TransformKind::Remux,
        b"output".to_vec(),
        16,
    )
    .unwrap();

    assert!(!store.publish_transform(stale).await.unwrap());
    assert_eq!(store.representation_binding("post").await, Some(next));
    assert_eq!(store.read_range("post", 0..6).await.unwrap(), None);
    store_fixture::discard(&root);
}

fn meta(label: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{label}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(5),
        duration_ms: Some(1_000),
    }
}
