use crate::tests::store_fixture::{discard, limits, spaced_store};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn rollback_refuses_to_overwrite_a_mutated_reservation() {
    let fixture = spaced_store("action-extension-mutation", limits(8, 0), 8);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    let action = fixture
        .store
        .reserve_action(&identity, 1, 2)
        .await
        .expect("valid test fixture");
    let extension = fixture
        .store
        .extend_action(&action, 6)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .resize_action(&action, 5)
        .await
        .expect("valid test fixture");

    fixture
        .store
        .rollback_action(extension)
        .await
        .expect_err("rollback must not overwrite intervening response mutation");
    fixture
        .store
        .reserve_action(&identity, 2, 4)
        .await
        .expect_err("the mutated five-byte reservation must remain intact");
    fixture.store.release_action(&action).await;
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
