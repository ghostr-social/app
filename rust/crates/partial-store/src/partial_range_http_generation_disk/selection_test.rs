use super::{load, save, StoredHttpGeneration, StoredHttpGenerationLoad};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{HttpGenerationKey, RequestSelection};

#[tokio::test]
async fn persisted_http_authority_preserves_the_request_selection_partition() {
    let tick = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ghostr-http-selection-{tick}"));
    tokio::fs::create_dir_all(&root).await.expect("directory");
    let path = root.join("generation.json");
    let key = HttpGenerationKey::try_new(
        "https://media.example/clip",
        EvidenceValidator::strong_etag("\"v1\""),
    )
    .expect("generation")
    .with_request_selection(Some(RequestSelection::new([7; 32])));
    let stored = StoredHttpGeneration {
        representation: "clip".into(),
        source: "https://media.example/clip".into(),
        key,
    };
    save(&path, &stored).await.expect("save");
    let StoredHttpGenerationLoad::Valid(restored) = load(&path).await.expect("read") else {
        panic!("valid generation")
    };
    assert_eq!(restored, stored);
    tokio::fs::remove_dir_all(root).await.expect("cleanup");
}
