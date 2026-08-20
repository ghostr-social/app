mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(unix)]
#[tokio::test]
async fn transient_generation_read_failure_preserves_resumable_bytes() {
    let root = store_fixture::temp_root("generation-transient-read");
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

    let used = Arc::new(Mutex::new(0));
    let reopened = store_fixture::plain_store(root.clone(), used.clone());
    reopened.load_existing().await.unwrap();
    let sidecar = root.join("clip.generation.json");
    let original = std::fs::metadata(&sidecar).unwrap().permissions();
    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o000)).unwrap();

    assert!(reopened.bind_representation(binding.clone()).await.is_err());
    assert_eq!(*used.lock().await, 4);
    assert_eq!(reopened.present_ranges("clip").await.unwrap(), vec![0..4]);

    std::fs::set_permissions(&sidecar, original).unwrap();
    reopened.bind_representation(binding).await.unwrap();
    assert_eq!(
        reopened.select_transfer(identity).await.unwrap(),
        Some(generation)
    );
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
