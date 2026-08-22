use crate::qoe::{load_playback_learning, save_playback_learning, QoeStats};
use ghostr_engine::watch_model::WatchModel;

#[tokio::test]
async fn interrupted_staging_never_replaces_the_last_complete_snapshot() {
    let root = crate::tests::support::temp_directory("watch-model-atomic");
    let path = root.join("qoe_stats.json");
    let staging = path.with_extension("json.tmp");
    let expected = QoeStats {
        completions: 7,
        ..QoeStats::default()
    };
    save_playback_learning(&path, &expected, &WatchModel::default())
        .await
        .unwrap();
    tokio::fs::write(&staging, b"interrupted raw-post")
        .await
        .unwrap();

    assert_eq!(load_playback_learning(&path).await.qoe, expected);
    save_playback_learning(&path, &expected, &WatchModel::default())
        .await
        .unwrap();
    assert!(!staging.exists());
    std::fs::remove_dir_all(root).ok();
}
