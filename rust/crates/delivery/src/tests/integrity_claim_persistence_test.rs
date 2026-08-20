use crate::manager::reliability::{load_catalog_evidence, save_catalog_evidence};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn mirror_claims_survive_the_production_evidence_snapshot() {
    let digest = "d".repeat(64);
    let path = std::env::temp_dir().join(format!("catalog-evidence-{}.json", std::process::id()));
    let mut catalog = Catalog::new();
    catalog.upsert(PostId::new("cached"), meta("cached", &digest));

    save_catalog_evidence(&path, &catalog.evidence_state())
        .await
        .unwrap();
    let persisted = load_catalog_evidence(&path).await;
    let mut restored = Catalog::new();
    restored.replace_evidence_state(persisted, 1);
    let failed = PostId::new("failed");
    let binding = restored.upsert(failed.clone(), meta("failed", &digest));
    let identity = binding
        .transfer("https://failed.example/video.mp4")
        .unwrap();

    assert_eq!(
        restored.quarantine_mirror_group(&identity, &digest, 2),
        vec![PostId::new("cached"), failed]
    );
    let _ = tokio::fs::remove_file(path).await;
}

fn meta(host: &str, digest: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{host}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: Some(digest.into()),
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    }
}
