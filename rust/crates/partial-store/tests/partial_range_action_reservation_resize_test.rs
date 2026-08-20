mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use store_fixture::{discard, limits, spaced_store};

#[tokio::test]
async fn observed_response_shrinks_the_exact_hard_reservation() {
    let fixture = spaced_store("action-resize", limits(8, 0), 8);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding.transfer("https://cdn.example/video").unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    let first = fixture.store.reserve_action(&identity, 1, 6).await.unwrap();

    fixture.store.resize_action(&first, 2).await.unwrap();
    let second = fixture.store.reserve_action(&identity, 2, 6).await.unwrap();
    fixture
        .store
        .resize_action(&first, 3)
        .await
        .expect_err("headers cannot expand the immutable launch grant");

    fixture.store.release_action(&first).await;
    fixture.store.release_action(&second).await;
    discard(&fixture.root);
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
