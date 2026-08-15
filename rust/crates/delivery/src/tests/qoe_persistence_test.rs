use crate::qoe::{load_qoe_stats, save_qoe_stats, QoeStats};

#[tokio::test]
async fn aggregate_learning_survives_a_restart_without_media_identity() {
    let root = std::env::temp_dir().join(format!("qoe-stats-{}", std::process::id()));
    let path = root.join("qoe_stats.json");
    tokio::fs::create_dir_all(&root).await.unwrap();
    let expected = QoeStats {
        first_frames: 2,
        startup_total_ms: 600,
        startup_max_ms: 400,
        ..QoeStats::default()
    };

    save_qoe_stats(&path, &expected).await.unwrap();
    let restored = load_qoe_stats(&path).await;

    assert_eq!(restored, expected);
    assert_eq!(restored.startup_eta_ms(), expected.startup_eta_ms());
    let json = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(!json.contains("clip"));
    std::fs::remove_dir_all(root).ok();
}
