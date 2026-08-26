use crate::tests::store_fixture::{discard, limits, spaced_store};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn stale_same_id_extension_cannot_shrink_a_new_action() {
    let fixture = spaced_store("action-extension-identity", limits(8, 0), 8);
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
    let stale = fixture
        .store
        .reserve_action(&identity, 1, 2)
        .await
        .expect("valid test fixture");
    let extension = fixture
        .store
        .extend_action(&stale, 4)
        .await
        .expect("valid test fixture");
    fixture.store.release_action(&stale).await;
    let current = fixture
        .store
        .reserve_action(&identity, 1, 4)
        .await
        .expect("valid test fixture");

    fixture
        .store
        .rollback_action(extension)
        .await
        .expect_err("the stale authority must not mutate the reused id");
    fixture
        .store
        .reserve_action(&identity, 2, 5)
        .await
        .expect_err("the current four-byte reservation must remain intact");
    fixture.store.release_action(&current).await;
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
