use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn clear_revokes_actions_and_releases_their_capacity() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "clear-actions",
        crate::tests::store_fixture::limits(8, 0),
        8,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    let stale = fixture
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .open_single_response_for_action(
            &identity,
            &stale,
            crate::tests::store_fixture::exact_response(8),
        )
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_single_response_for_action(&identity, &stale, 0, b"half")
        .await
        .expect("valid test fixture");

    fixture.store.clear().await.expect("valid test fixture");

    assert!(!stale.is_active());
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    let current = fixture
        .store
        .reserve_action(&identity, 2, 8)
        .await
        .expect("valid test fixture");
    assert!(!fixture
        .store
        .write_single_response_for_action(&identity, &stale, 4, b"late")
        .await
        .expect("valid test fixture"));
    fixture.store.release_action(&current).await;
    crate::tests::store_fixture::discard(&fixture.root);
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
