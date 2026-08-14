//! A gateway demand signal promotes the playing post's missing bytes
//! to playback-critical authority when the adaptive plan otherwise withholds them.

mod delivery_fixture;
mod range_fixture;

use delivery_fixture::demand;
use delivery_fixture::items::{focus_now, sized_item, unsized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::{ByteRange, EngineParams};
use range_fixture::reject::serve_failing;

#[tokio::test]
async fn delivery_manager_promotes_demanded_bytes_to_emergency() {
    let origin = serve_recording("origin", media_body(), hit_log()).await;
    let hungry = serve_failing().await;
    let harness = start_harness("ghostr-delivery-demand", short_head_options());

    // The unavailable second post adds no useful allocation. The current
    // reserve is already covered, so its unrequested tail stays absent.
    harness.handle.update_focus(focus_now(
        vec![
            sized_item("aa11", &origin, 16, 4_000),
            unsized_item("bb22", &hungry),
        ],
        0,
        0,
    ));
    wait_for_ranges(&harness.store, "aa11", &[(0, 4)]).await;
    let missing = harness
        .store
        .missing_within("aa11", 4..16)
        .await
        .expect("gaps");
    assert_eq!(
        missing,
        vec![4..16],
        "the plan withholds unrequested tail bytes"
    );

    let _demand = demand::blocked(&harness, "aa11", ByteRange::new(8, 16)).await;

    wait_for_ranges(&harness.store, "aa11", &[(8, 16)]).await;
    std::fs::remove_dir_all(&harness.root).ok();
}

/// A one-second head budget so the 16-byte file splits head/tail.
fn short_head_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 4,
            ..base_params()
        },
        ..DeliveryOptions::default()
    }
}
