use crate::qoe::{load_playback_learning, save_playback_learning, QoeStats};
use ghostr_engine::watch_model::{
    WatchContext, WatchKey, WatchModel, WatchSample, WatchSampleKind,
};

#[tokio::test]
async fn learned_watch_predictions_survive_an_atomic_privacy_safe_restart() {
    let root = crate::tests::support::temp_directory("watch-model-restart");
    let path = root.join("qoe_stats.json");
    let raw = "raw-post|https://private.example/secret.mp4";
    let context = WatchContext::new(WatchKey::digest(raw), Some(20_000));
    let mut model = WatchModel::default();
    for observed_at_ms in 1..=8 {
        model.observe(&WatchSample::new(
            context.clone(),
            1_000,
            WatchSampleKind::Abandoned,
            observed_at_ms,
        ));
    }
    let expected = model.predict(&context, 20).p50_ms();

    save_playback_learning(&path, &QoeStats::default(), &model)
        .await
        .expect("valid test fixture");
    let restored = load_playback_learning(&path).await;
    let json = tokio::fs::read_to_string(&path)
        .await
        .expect("valid test fixture");

    assert_eq!(restored.watch.predict(&context, 20).p50_ms(), expected);
    assert_eq!(restored.watch.revision(), model.revision());
    assert!(!json.contains("raw-post"));
    assert!(!json.contains("private.example"));
    std::fs::remove_dir_all(root).ok();
}
