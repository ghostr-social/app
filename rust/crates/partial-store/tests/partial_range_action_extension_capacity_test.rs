use crate::tests::store_fixture::{discard, limits, spaced_store};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn action_extension_reserves_only_its_incremental_capacity() {
    let fixture = spaced_store("action-extension-capacity", limits(8, 0), 8);
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

    assert_eq!(extension.additional_bytes(), 4);
    fixture
        .store
        .reserve_action(&identity, 2, 3)
        .await
        .expect_err("the four-byte increment must consume real capacity");
    extension.commit();
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
