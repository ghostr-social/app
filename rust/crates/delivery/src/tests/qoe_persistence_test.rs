use crate::qoe::{load_playback_learning, save_playback_learning, QoeStats};
use ghostr_engine::watch_model::WatchModel;

#[tokio::test]
async fn aggregate_learning_survives_a_restart_without_media_identity() {
    let root = std::env::temp_dir().join(format!("qoe-stats-{}", std::process::id()));
    let path = root.join("qoe_stats.json");
    tokio::fs::create_dir_all(&root)
        .await
        .expect("valid test fixture");
    let expected = QoeStats {
        first_frames: 2,
        startup_total_ms: 600,
        startup_max_ms: 400,
        ..QoeStats::default()
    };

    save_playback_learning(&path, &expected, &WatchModel::default())
        .await
        .expect("valid test fixture");
    let restored = load_playback_learning(&path).await.qoe;

    assert_eq!(restored, expected);
    assert_eq!(restored.startup_eta_ms(), expected.startup_eta_ms());
    let json = tokio::fs::read_to_string(&path)
        .await
        .expect("valid test fixture");
    assert!(!json.contains("clip"));
    std::fs::remove_dir_all(root).ok();
}
