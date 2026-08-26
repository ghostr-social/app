#![cfg(unix)]

use crate::partial_range_store::ResponseOpenResult;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn failed_stage_cleanup_releases_future_capacity_and_retries_owned_bytes() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "action-cleanup-failure",
        crate::tests::store_fixture::limits(12, 0),
        12,
    );
    let mut catalog = Catalog::new();
    let first_binding = catalog.upsert(PostId::new("post"), meta("post"));
    let first = first_binding
        .transfer("https://cdn.example/post")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(first_binding)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .select_transfer(first.clone())
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range("post", 0, b"old!")
        .await
        .expect("valid test fixture");
    let action = fixture
        .store
        .reserve_action(&first, 1, 8)
        .await
        .expect("valid test fixture");
    assert_eq!(
        fixture
            .store
            .open_single_response_for_action(
                &first,
                &action,
                crate::tests::store_fixture::exact_response(8),
            )
            .await
            .expect("valid test fixture"),
        ResponseOpenResult::Opened
    );
    fixture
        .store
        .write_single_response_for_action(&first, &action, 0, b"new!")
        .await
        .expect("valid test fixture");
    std::fs::create_dir(fixture.root.join("post.response.commit")).expect("valid test fixture");
    fixture.store.release_action(&action).await;

    let second_binding = catalog.upsert(PostId::new("other"), meta("other"));
    let second = second_binding
        .transfer("https://cdn.example/other")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(second_binding)
        .await
        .expect("valid test fixture");
    let other = fixture
        .store
        .reserve_action(&second, 2, 4)
        .await
        .expect("valid test fixture");
    fixture.store.release_action(&other).await;
    std::fs::remove_dir(fixture.root.join("post.response.commit")).expect("valid test fixture");

    let retry = fixture
        .store
        .reserve_action(&first, 3, 8)
        .await
        .expect("valid test fixture");
    assert_eq!(*fixture.used_bytes.lock().await, 4);
    fixture.store.release_action(&retry).await;
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
