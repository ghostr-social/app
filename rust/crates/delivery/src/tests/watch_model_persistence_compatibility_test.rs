use crate::qoe::{load_playback_learning, QoeStats};

#[tokio::test]
async fn legacy_qoe_files_restore_aggregates_with_a_fresh_watch_model() {
    let root = crate::tests::support::temp_directory("watch-model-compatibility");
    let path = root.join("qoe_stats.json");
    let legacy = QoeStats {
        completions: 7,
        ..QoeStats::default()
    };
    tokio::fs::create_dir_all(&root).await.unwrap();
    tokio::fs::write(&path, serde_json::to_vec(&legacy).unwrap())
        .await
        .unwrap();

    let restored = load_playback_learning(&path).await;
    assert_eq!(restored.qoe.completions, 7);
    assert_eq!(restored.watch.revision(), 0);

    std::fs::remove_dir_all(root).ok();
}
