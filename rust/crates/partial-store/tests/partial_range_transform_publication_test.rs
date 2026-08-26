use crate::partial_range_store::{TransformFence, TransformPublication};
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn transformed_bytes_replace_the_exact_input_and_survive_restart() {
    let root = crate::tests::store_fixture::temp_root("partial-transform-publish");
    let input = binding("https://origin.example/input.mp4");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .bind_representation(input.clone())
        .await
        .expect("valid test fixture");
    store
        .write_range("post", 0, b"input")
        .await
        .expect("valid test fixture");
    store
        .set_total_len("post", 5)
        .await
        .expect("valid test fixture");
    store
        .finalize("post", None)
        .await
        .expect("valid test fixture");
    let before = store
        .media_snapshot("post")
        .await
        .expect("valid test fixture");
    let publication = TransformPublication::try_new(
        TransformFence::new(input.clone(), before.revision()),
        TransformKind::Remux,
        b"output".to_vec(),
        16,
    )
    .expect("valid test fixture");

    assert!(store
        .publish_transform(publication)
        .await
        .expect("valid test fixture"));
    let after = store
        .media_snapshot("post")
        .await
        .expect("valid test fixture");
    assert!(after
        .binding()
        .expect("valid test fixture")
        .derives_from(&input));
    assert_eq!(
        store
            .read_range("post", 0..6)
            .await
            .expect("valid test fixture"),
        Some(b"output".to_vec())
    );
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(input.clone())
        .await
        .expect("valid test fixture");
    let restored = reopened
        .media_snapshot("post")
        .await
        .expect("valid test fixture");
    assert!(restored
        .binding()
        .expect("valid test fixture")
        .derives_from(&input));
    assert_eq!(
        reopened
            .read_range("post", 0..6)
            .await
            .expect("valid test fixture"),
        Some(b"output".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
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
