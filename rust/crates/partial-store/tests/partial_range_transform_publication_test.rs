mod store_fixture;

use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::{TransformFence, TransformPublication};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn transformed_bytes_replace_the_exact_input_and_survive_restart() {
    let root = store_fixture::temp_root("partial-transform-publish");
    let input = binding("https://origin.example/input.mp4");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.bind_representation(input.clone()).await.unwrap();
    store.write_range("post", 0, b"input").await.unwrap();
    store.set_total_len("post", 5).await.unwrap();
    store.finalize("post", None).await.unwrap();
    let before = store.media_snapshot("post").await.unwrap();
    let publication = TransformPublication::try_new(
        TransformFence::new(input.clone(), before.revision()),
        TransformKind::Remux,
        b"output".to_vec(),
        16,
    )
    .unwrap();

    assert!(store.publish_transform(publication).await.unwrap());
    let after = store.media_snapshot("post").await.unwrap();
    assert!(after.binding().unwrap().derives_from(&input));
    assert_eq!(
        store.read_range("post", 0..6).await.unwrap(),
        Some(b"output".to_vec())
    );
    drop(store);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(input.clone()).await.unwrap();
    let restored = reopened.media_snapshot("post").await.unwrap();
    assert!(restored.binding().unwrap().derives_from(&input));
    assert_eq!(
        reopened.read_range("post", 0..6).await.unwrap(),
        Some(b"output".to_vec())
    );
    store_fixture::discard(&root);
}

fn binding(source: &str) -> ghostr_engine::representation::RepresentationBinding {
    Catalog::new().upsert(
        PostId::new("post"),
        VideoMeta {
            urls: vec![source.into()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(5),
            duration_ms: Some(1_000),
        },
    )
}
