//! A playing current item fills only its safe buffer frontier, then
//! yields the serial slot to ahead startup before the current EOF.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::media::{hit_log, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_delivery::delivery_events::DeliveryPlayback;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DataUsageLevel, EngineParams, PostId};
use std::time::Duration;

#[tokio::test]
async fn safe_current_buffer_yields_to_ahead_before_current_eof() {
    let current = serve_recording("current", vec![1; 80], hit_log()).await;
    let ahead = serve_recording("ahead", vec![2; 16], hit_log()).await;
    let harness = start_harness("ghostr-delivery-playback-ahead", options());
    let current_item = sized_item("aa11", &current, 80, 80_000);
    let ahead_item = sized_item("bb22", &ahead, 16, 16_000);
    seed_range(&harness.store, &current_item, 0, &[1]).await;

    harness
        .handle
        .update_focus(focus_now(vec![current_item, ahead_item], 0, 5_000));
    harness.handle.report_playback(playback());

    wait_for_ranges(&harness.store, "bb22", &[(0, 1)]).await;
    let current_ranges = harness.store.present_ranges("aa11").await.unwrap();
    assert!(current_ranges.iter().all(|range| range.end < 80));
    assert!(current_ranges.iter().any(|range| range.end >= 5));
    std::fs::remove_dir_all(&harness.root).ok();
}

fn playback() -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new("aa11"), 1),
        sequence: PlaybackObservationSequence::new(1),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(1),
            1_000,
            PlaybackPhase::Playing,
        )
        .unwrap(),
    }
}

fn options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            head_seconds: 1,
            chunk_bytes: 4,
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
