use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn failed_policy_eviction_cleans_its_uncommitted_payload_before_returning() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "policy-failure-cleanup",
        crate::tests::store_fixture::limits(20, 0),
        20,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("valid test fixture");
    fixture
        .store
        .set_total_len("clip", 12)
        .await
        .expect("valid test fixture");
    let manifest = fixture.root.join("clip.ranges.json");
    let stable = tokio::fs::read(&manifest)
        .await
        .expect("valid test fixture");
    tokio::fs::remove_file(&manifest)
        .await
        .expect("valid test fixture");
    tokio::fs::create_dir(&manifest)
        .await
        .expect("valid test fixture");

    fixture
        .store
        .evict_ranges("clip", core::slice::from_ref(&(4..8)))
        .await
        .expect_err("scenario must fail");

    assert!(!fixture.root.join("clip.part.evict").exists());
    assert!(!fixture.root.join("clip.evict.intent").exists());
    assert!(!fixture.root.join("clip.evict.intent.tmp").exists());
    tokio::fs::remove_dir(&manifest)
        .await
        .expect("valid test fixture");
    tokio::fs::write(&manifest, stable)
        .await
        .expect("valid test fixture");
    assert_eq!(fixture.store.used_bytes().await, 12);
    let action = fixture
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("valid test fixture");
    fixture.store.release_action(&action).await;
    crate::tests::store_fixture::discard(&fixture.root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(12),
        duration_ms: Some(1_000),
    }
}
