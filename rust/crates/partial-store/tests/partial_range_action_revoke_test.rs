use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn revoke_keeps_headroom_reserved_until_terminal_cleanup() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "action-revoke",
        crate::tests::store_fixture::limits(8, 0),
        8,
    );
    let mut catalog = Catalog::new();
    let first_binding = catalog.upsert(PostId::new("first"), meta("first"));
    let first = first_binding
        .transfer("https://cdn.example/first")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(first_binding)
        .await
        .expect("valid test fixture");
    let stale = fixture
        .store
        .reserve_action(&first, 1, 8)
        .await
        .expect("valid test fixture");
    let changes = fixture.store.capacity_changes();

    stale.revoke();

    assert!(changes.has_changed().expect("valid test fixture"));
    let next_binding = catalog.upsert(PostId::new("next"), meta("next"));
    let next = next_binding
        .transfer("https://cdn.example/next")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(next_binding)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .reserve_action(&next, 2, 8)
        .await
        .expect_err("revoked work still owns its physical cleanup budget");
    fixture.store.release_action(&stale).await;
    let current = fixture
        .store
        .reserve_action(&next, 2, 8)
        .await
        .expect("valid test fixture");
    fixture.store.release_action(&current).await;
    crate::tests::store_fixture::discard(&fixture.root);
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
