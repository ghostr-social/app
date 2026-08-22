mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn persisted_weak_validator_never_authorizes_sparse_continuation() {
    let root = store_fixture::temp_root("generation-validation");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding.transfer(&meta().urls[0]).unwrap();
    let generation = SourceGeneration::try_new(&meta().urls[0], "\"valid\"", 8).unwrap();
    let first = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    first.bind_representation(binding.clone()).await.unwrap();
    first.select_transfer(identity.clone()).await.unwrap();
    first
        .accept_generation(&identity, generation.clone())
        .await
        .unwrap();
    first
        .write_range_for_generation_if_current(&identity, &generation, 0, b"part")
        .await
        .unwrap();
    drop(first);

    let invalid = serde_json::json!({
        "representation": binding.representation().fingerprint(),
        "source": meta().urls[0],
        "generation": {
            "final_url": meta().urls[0],
            "strong_etag": "W/\"weak\"",
            "total_bytes": 8
        }
    });
    tokio::fs::write(
        root.join("clip.generation.json"),
        serde_json::to_vec(&invalid).unwrap(),
    )
    .await
    .unwrap();

    let used = Arc::new(Mutex::new(0));
    let reopened = store_fixture::plain_store(root.clone(), used.clone());
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();

    assert_eq!(reopened.select_transfer(identity).await.unwrap(), None);
    assert!(reopened.present_ranges("clip").await.unwrap().is_empty());
    assert_eq!(*used.lock().await, 0);
    store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
