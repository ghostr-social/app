use crate::tests::store_fixture::{discard, limits, spaced_store};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn action_reservations_are_exclusive_and_release_exactly_once() {
    let fixture = spaced_store("action-reservation", limits(8, 0), 8);
    let identity = identity();
    fixture
        .store
        .bind_representation(binding())
        .await
        .expect("valid test fixture");
    let first = fixture
        .store
        .reserve_action(&identity, 1, 6)
        .await
        .expect("first reservation");
    fixture
        .store
        .reserve_action(&identity, 2, 3)
        .await
        .expect_err("outstanding grants consume the hard capacity");

    fixture.store.release_action(&first).await;
    fixture.store.release_action(&first).await;
    fixture
        .store
        .reserve_action(&identity, 2, 3)
        .await
        .expect("released capacity is reusable");
    discard(&fixture.root);
}

fn binding() -> ghostr_engine::representation::RepresentationBinding {
    let mut catalog = Catalog::new();
    catalog.upsert(PostId::new("post"), meta())
}

fn identity() -> ghostr_engine::representation::TransferIdentity {
    binding()
        .transfer("https://cdn.example/video")
        .expect("valid test fixture")
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
