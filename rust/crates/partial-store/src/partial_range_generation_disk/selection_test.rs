use super::{load, save, StoredGeneration};
use ghostr_engine::representation::{RequestSelection, SourceGeneration};

#[tokio::test]
async fn persisted_generation_preserves_the_request_selection_partition() {
    let tick = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ghostr-request-selection-{tick}"));
    tokio::fs::create_dir_all(&root).await.expect("directory");
    let path = root.join("generation.json");
    let selected = RequestSelection::new([7; 32]);
    let generation = SourceGeneration::try_new("https://media.example/clip", "\"v1\"", 8)
        .expect("generation")
        .with_request_selection(Some(selected));
    let stored = StoredGeneration {
        representation: "clip".into(),
        source: "https://media.example/clip".into(),
        generation: generation.clone(),
    };
    save(&path, &stored).await.expect("save");
    let restored = load(&path).await.expect("read").expect("generation");
    assert_eq!(restored.generation, generation);
    assert_ne!(
        restored.generation,
        generation.with_request_selection(Some(RequestSelection::new([8; 32])))
    );
    tokio::fs::remove_dir_all(root).await.expect("cleanup");
}
