mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use store_fixture::{discard, limits, spaced_store};

#[tokio::test]
async fn rollback_restores_the_exact_incremental_capacity() {
    let fixture = spaced_store("action-extension-rollback", limits(8, 0), 8);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding.transfer("https://cdn.example/video").unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    let action = fixture.store.reserve_action(&identity, 1, 2).await.unwrap();
    let extension = fixture.store.extend_action(&action, 6).await.unwrap();

    fixture.store.rollback_action(extension).await.unwrap();

    let peer = fixture.store.reserve_action(&identity, 2, 6).await.unwrap();
    fixture.store.release_action(&action).await;
    fixture.store.release_action(&peer).await;
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
