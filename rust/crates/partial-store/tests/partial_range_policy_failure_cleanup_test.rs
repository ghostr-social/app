mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn failed_policy_eviction_cleans_its_uncommitted_payload_before_returning() {
    let fixture =
        store_fixture::spaced_store("policy-failure-cleanup", store_fixture::limits(20, 0), 20);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding.transfer("https://cdn.example/video").unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    fixture
        .store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .unwrap();
    fixture.store.set_total_len("clip", 12).await.unwrap();
    let manifest = fixture.root.join("clip.ranges.json");
    let stable = tokio::fs::read(&manifest).await.unwrap();
    tokio::fs::remove_file(&manifest).await.unwrap();
    tokio::fs::create_dir(&manifest).await.unwrap();

    fixture
        .store
        .evict_ranges("clip", std::slice::from_ref(&(4..8)))
        .await
        .unwrap_err();

    assert!(!fixture.root.join("clip.part.evict").exists());
    assert!(!fixture.root.join("clip.evict.intent").exists());
    assert!(!fixture.root.join("clip.evict.intent.tmp").exists());
    tokio::fs::remove_dir(&manifest).await.unwrap();
    tokio::fs::write(&manifest, stable).await.unwrap();
    assert_eq!(fixture.store.used_bytes().await, 12);
    let action = fixture.store.reserve_action(&identity, 1, 8).await.unwrap();
    fixture.store.release_action(&action).await;
    store_fixture::discard(&fixture.root);
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
