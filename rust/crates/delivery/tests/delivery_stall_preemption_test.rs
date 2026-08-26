//! A real playback emergency takes the slot from speculative ahead work.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::host_hol::SlowHost;
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryPlayback;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{EngineParams, PostId};

#[tokio::test]
async fn network_stall_preempts_a_still_wanted_ahead_transfer() {
    let current = SlowHost::serve().await;
    let ahead = SlowHost::serve().await;
    let harness = start_harness("ghostr-stall-preemption", options());
    let current_item = sized_item("current", &current.localhost_url("video"), 64, 8_000);
    let ahead_item = sized_item("ahead", &ahead.url("video"), 64, 8_000);
    seed_range(&harness.store, &current_item, 0, &[1; 32]).await;
    harness
        .handle
        .update_focus(focus_now(vec![current_item, ahead_item], 0, 0));
    tokio::time::timeout(Duration::from_secs(1), ahead.wait_started())
        .await
        .expect("ahead speculation starts");

    harness.handle.report_playback(stalled_playback());

    tokio::time::timeout(Duration::from_millis(300), current.wait_started())
        .await
        .expect("stalled current post takes the slot");
    assert!(
        tokio::time::timeout(Duration::from_millis(150), ahead.wait_started())
            .await
            .is_err(),
        "emergency playback must not reopen a speculative exploration slot"
    );
    current.release();
    ahead.release();
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn stalled_playback() -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new("current"), 1),
        sequence: PlaybackObservationSequence::new(1),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(1),
            1_000,
            PlaybackPhase::NetworkStalled,
        )
        .expect("valid test fixture"),
    }
}

fn options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            balanced_concurrency: 2,
            chunk_bytes: 32,
            ..base_params()
        },
        ..DeliveryOptions::default()
    }
}
