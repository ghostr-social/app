mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::playback::playing;
use delivery_fixture::{start_harness_at, temp_directory};
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::qoe::load_playback_learning;
use std::time::Duration;

const FIRST: &str = "private-watch-a";
const SECOND: &str = "private-watch-b";
const SOURCE: &str = "http://127.0.0.1:9/private-watch-source";

#[tokio::test]
async fn manager_restart_replays_learned_watch_evidence_into_real_decisions() {
    let root = temp_directory("delivery-watch-restart");
    let first = start_harness_at(root.clone(), DeliveryOptions::default());
    first.handle.update_focus(focus(0, 0));
    let (cold_seed, _) = wait_for_epoch(&first.handle, 0).await;
    first
        .handle
        .report_playback(playing(FIRST, Duration::from_millis(1_000)));
    wait_for_playback(&first.handle).await;
    first.handle.update_focus(focus(1, 2_000));
    let learned = wait_for_learning(&root.join("qoe_stats.json")).await;
    drop(first);

    let restarted = start_harness_at(root.clone(), DeliveryOptions::default());
    restarted.handle.update_focus(focus(0, 0));
    let (learned_seed, json) = wait_for_epoch(&restarted.handle, learned).await;

    assert_ne!(cold_seed, learned_seed);
    assert!(json.contains("play_start_p95_ms"));
    assert!(!json.contains(FIRST) && !json.contains(SECOND) && !json.contains(SOURCE));
    restarted.handle.clear().await.unwrap();
    std::fs::remove_dir_all(root).ok();
}

fn focus(index: usize, watch_ms: u64) -> ghostr_delivery::delivery_events::DeliveryFocus {
    focus_now(
        vec![
            sized_item(FIRST, SOURCE, 16, 8_000),
            sized_item(SECOND, SOURCE, 16, 8_000),
        ],
        index,
        watch_ms,
    )
}

async fn wait_for_playback(handle: &DeliveryHandle) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while handle.playback_admission_snapshot().counters().accepted() == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("accepted playback");
}

async fn wait_for_learning(path: &std::path::Path) -> u64 {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let epoch = load_playback_learning(path).await.watch.change_epoch();
            if epoch > 0 {
                break epoch;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("persisted WatchModel")
}

async fn wait_for_epoch(handle: &DeliveryHandle, expected: u64) -> (u64, String) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            for record in handle.decision_history().records.iter().rev() {
                let value = serde_json::to_value(record).unwrap();
                if value.pointer("/warp_decision/planner_replay_capsule/context/epochs/model")
                    == Some(&expected.into())
                {
                    return (
                        record
                            .warp_decision
                            .as_ref()
                            .unwrap()
                            .search
                            .common_random_seed,
                        value.to_string(),
                    );
                }
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("decision with model epoch")
}
