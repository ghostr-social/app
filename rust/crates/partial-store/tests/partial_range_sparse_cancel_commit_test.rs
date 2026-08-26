use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn cancelling_a_sparse_action_durably_keeps_its_coherent_prefix() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "sparse-cancel-commit",
        crate::tests::store_fixture::limits(8, 0),
        8,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding
        .transfer("https://cdn.example/clip")
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(identity.source().as_str(), "\"g\"", 8)
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    let action = fixture
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .open_sparse_response(&identity, &action, generation.clone(), ByteRange::new(0, 8))
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &action, 0, b"abcd")
        .await
        .expect("valid test fixture");

    assert!(persisted_intervals(&fixture.root).is_empty());
    action.revoke();
    fixture.store.release_action(&action).await;
    assert_eq!(persisted_intervals(&fixture.root), vec![(0, 4)]);

    let reopened = crate::tests::store_fixture::reopened(&fixture);
    reopened
        .store
        .load_existing()
        .await
        .expect("valid test fixture");
    assert_eq!(
        reopened
            .store
            .read_range("clip", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"abcd".to_vec())
    );
    crate::tests::store_fixture::discard(&fixture.root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/clip".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}

fn persisted_intervals(root: &std::path::Path) -> Vec<(u64, u64)> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("clip.ranges.json")).expect("valid test fixture"),
    )
    .expect("valid test fixture");
    value["intervals"]
        .as_array()
        .expect("valid test fixture")
        .iter()
        .map(|item| {
            (
                item["start"].as_u64().expect("valid test fixture"),
                item["end"].as_u64().expect("valid test fixture"),
            )
        })
        .collect()
}
