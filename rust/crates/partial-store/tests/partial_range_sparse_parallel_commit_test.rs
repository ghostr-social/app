mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn one_sparse_action_never_commits_its_siblings_provisional_bytes() {
    let fixture =
        store_fixture::spaced_store("sparse-parallel-commit", store_fixture::limits(8, 0), 8);
    let (binding, identity) = identity();
    let generation = SourceGeneration::try_new(identity.source().as_str(), "\"g\"", 8).unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    let first = fixture.store.reserve_action(&identity, 1, 4).await.unwrap();
    let second = fixture.store.reserve_action(&identity, 2, 4).await.unwrap();
    for (action, range) in [
        (&first, ByteRange::new(0, 4)),
        (&second, ByteRange::new(4, 8)),
    ] {
        fixture
            .store
            .open_sparse_response(&identity, action, generation.clone(), range)
            .await
            .unwrap();
    }
    assert!(fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &first, 0, b"abcd")
        .await
        .unwrap());
    assert!(fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &second, 4, b"efgh")
        .await
        .unwrap());

    assert!(fixture
        .store
        .finish_sparse_response(&identity, &generation, &first)
        .await
        .unwrap());
    assert_eq!(persisted_intervals(&fixture.root), vec![(0, 4)]);
    assert_eq!(
        fixture.store.read_range("clip", 0..8).await.unwrap(),
        Some(b"abcdefgh".to_vec())
    );

    assert!(fixture
        .store
        .finish_sparse_response(&identity, &generation, &second)
        .await
        .unwrap());
    assert_eq!(persisted_intervals(&fixture.root), vec![(0, 4), (4, 8)]);
    assert_eq!(
        fixture.store.present_ranges("clip").await.unwrap(),
        vec![0..8]
    );
    fixture.store.release_action(&first).await;
    fixture.store.release_action(&second).await;
    store_fixture::discard(&fixture.root);
}

fn identity() -> (
    ghostr_engine::representation::RepresentationBinding,
    ghostr_engine::representation::TransferIdentity,
) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(
        PostId::new("clip"),
        VideoMeta {
            urls: vec!["https://cdn.example/clip".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(8),
            duration_ms: Some(1_000),
        },
    );
    let identity = binding.transfer("https://cdn.example/clip").unwrap();
    (binding, identity)
}

fn persisted_intervals(root: &std::path::Path) -> Vec<(u64, u64)> {
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("clip.ranges.json")).unwrap())
            .unwrap();
    value["intervals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["start"].as_u64().unwrap(),
                item["end"].as_u64().unwrap(),
            )
        })
        .collect()
}
