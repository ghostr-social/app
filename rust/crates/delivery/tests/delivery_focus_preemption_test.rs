//! A focus jump takes the serial slot from still-relevant old work.

mod delivery_fixture;
mod range_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use ghostr_engine::{DataUsageLevel, EngineParams};
use range_fixture::stall::serve_stalling_signaled;

#[tokio::test]
async fn a_jump_preempts_old_work_even_when_the_old_post_remains_behind() {
    let (slow, started) = serve_stalling_signaled(media_body()[..4].to_vec(), 16).await;
    let fast_hits = hit_log();
    let fast = serve_recording("fast", media_body(), std::sync::Arc::clone(&fast_hits)).await;
    let harness = start_harness("ghostr-focus-preemption", options());
    harness.handle.update_focus(window(&slow, &fast, 0));
    tokio::time::timeout(Duration::from_secs(1), started)
        .await
        .expect("old focus starts")
        .expect("old origin signal");

    harness.handle.update_focus(window(&slow, &fast, 1));

    tokio::time::timeout(Duration::from_millis(300), async {
        while !hits(&fast_hits)
            .iter()
            .any(|hit| hit.starts_with("fast:GET"))
        {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("new focus owns the serial slot");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn window(
    slow: &str,
    fast: &str,
    current: usize,
) -> ghostr_delivery::delivery_events::DeliveryFocus {
    focus_now(
        vec![
            sized_item("old", slow, 16, 1_000),
            sized_item("current", fast, 16, 1_000),
        ],
        current,
        0,
    )
}

fn options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 16,
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
