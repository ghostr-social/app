use crate::qoe::{load_playback_learning, QoeStats};

#[tokio::test]
async fn unsupported_learning_envelope_fails_closed() {
    let root = crate::tests::support::temp_directory("watch-model-version");
    let path = root.join("qoe_stats.json");
    tokio::fs::write(
        &path,
        br#"{"version":2,"qoe":{"completions":7},"watch":{}}"#,
    )
    .await
    .expect("valid test fixture");

    let restored = load_playback_learning(&path).await;

    assert_eq!(restored.qoe, QoeStats::default());
    assert_eq!(restored.watch.revision(), 0);
    std::fs::remove_dir_all(root).ok();
}
