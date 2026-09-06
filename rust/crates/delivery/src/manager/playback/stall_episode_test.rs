use super::test_support::{update, worker};
use ghostr_engine::concurrency::AdaptiveConcurrency;
use ghostr_engine::playback::PlaybackPhase;

#[tokio::test]
async fn repeated_stall_telemetry_is_one_capacity_setback_per_playback_episode() {
    let (mut worker, root) = worker().await;
    worker.concurrency = AdaptiveConcurrency::new(4, 4);
    worker.apply_playback(&update(1, 1, PlaybackPhase::NetworkStalled));
    assert_eq!(worker.concurrency.limit(), 3);
    for sequence in 2..20 {
        worker.apply_playback(&update(1, sequence, PlaybackPhase::NetworkStalled));
    }
    assert_eq!(
        worker.concurrency.limit(),
        3,
        "one stall is not 19 independent setbacks"
    );
    worker.apply_playback(&update(1, 20, PlaybackPhase::Playing));
    worker.apply_playback(&update(1, 21, PlaybackPhase::NetworkStalled));
    assert_eq!(
        worker.concurrency.limit(),
        2,
        "a new stall remains a real setback"
    );
    worker.apply_playback(&update(1, 19, PlaybackPhase::Playing));
    worker.apply_playback(&update(1, 22, PlaybackPhase::NetworkStalled));
    assert_eq!(
        worker.concurrency.limit(),
        2,
        "stale recovery cannot split an episode"
    );
    worker.apply_playback(&update(2, 1, PlaybackPhase::NetworkStalled));
    assert_eq!(
        worker.concurrency.limit(),
        1,
        "a new backend epoch is a new episode"
    );
    drop(worker);
    let _ = tokio::fs::remove_dir_all(root).await;
}
