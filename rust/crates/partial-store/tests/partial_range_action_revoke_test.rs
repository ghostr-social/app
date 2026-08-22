mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn revoke_keeps_headroom_reserved_until_terminal_cleanup() {
    let fixture = store_fixture::spaced_store("action-revoke", store_fixture::limits(8, 0), 8);
    let mut catalog = Catalog::new();
    let first_binding = catalog.upsert(PostId::new("first"), meta("first"));
    let first = first_binding.transfer("https://cdn.example/first").unwrap();
    fixture
        .store
        .bind_representation(first_binding)
        .await
        .unwrap();
    let stale = fixture.store.reserve_action(&first, 1, 8).await.unwrap();
    let changes = fixture.store.capacity_changes();

    stale.revoke();

    assert!(changes.has_changed().unwrap());
    let next_binding = catalog.upsert(PostId::new("next"), meta("next"));
    let next = next_binding.transfer("https://cdn.example/next").unwrap();
    fixture
        .store
        .bind_representation(next_binding)
        .await
        .unwrap();
    fixture
        .store
        .reserve_action(&next, 2, 8)
        .await
        .expect_err("revoked work still owns its physical cleanup budget");
    fixture.store.release_action(&stale).await;
    let current = fixture.store.reserve_action(&next, 2, 8).await.unwrap();
    fixture.store.release_action(&current).await;
    store_fixture::discard(&fixture.root);
}

fn meta(name: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://cdn.example/{name}")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
