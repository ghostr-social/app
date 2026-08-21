use crate::qoe::{load_playback_learning, QoeStats};

#[tokio::test]
async fn corrupt_watch_state_is_dropped_without_discarding_valid_qoe() {
    let root = crate::tests::support::temp_directory("watch-model-corrupt");
    let path = root.join("qoe_stats.json");
    tokio::fs::write(
        &path,
        br#"{"version":1,"qoe":{"completions":7},"watch":"raw-post"}"#,
    )
    .await
    .unwrap();

    let restored = load_playback_learning(&path).await;

    assert_eq!(restored.qoe.completions, 7);
    assert_eq!(restored.watch.revision(), 0);
    assert_ne!(restored.qoe, QoeStats::default());
    std::fs::remove_dir_all(root).ok();
}
