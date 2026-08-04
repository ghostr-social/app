//! A gateway demand signal promotes the playing post's missing bytes
//! to the emergency tier; without it, hunger mode never fetches them.

mod range_fixture;
mod support;

use range_fixture::reject::serve_failing;
use rust_lib_ghostr::engine::{ByteRange, EngineParams, PostId};
use rust_lib_ghostr::video::playback_demand::DemandSignal;
use support::delivery::{base_params, start_harness, DeliveryOptions};
use support::delivery_items::{focus_now, sized_item, unsized_item};
use support::delivery_media::{hit_log, media_body, serve_recording};
use support::delivery_wait::wait_for_ranges;

#[tokio::test]
async fn delivery_manager_promotes_demanded_bytes_to_emergency() {
    let origin = serve_recording("origin", media_body(), hit_log()).await;
    let hungry = serve_failing().await;
    let harness = start_harness("ghostr-delivery-demand", short_head_options());

    // The failing second post keeps the startability target unmet, so
    // the uncommitted current post is owed nothing beyond its head.
    harness.handle.update_focus(focus_now(
        vec![
            sized_item("aa11", &origin, 16, 4_000),
            unsized_item("bb22", &hungry),
        ],
        0,
        0,
    ));
    wait_for_ranges(&harness.store, "aa11", &[(0, 4)]).await;
    let missing = harness.store.missing_within("aa11", 4..16).await.expect("gaps");
    assert_eq!(missing, vec![4..16], "hunger withholds the tail");

    harness.demand.emit(DemandSignal {
        post: PostId::new("aa11"),
        range: ByteRange::new(8, 16),
    });

    wait_for_ranges(&harness.store, "aa11", &[(8, 16)]).await;
    std::fs::remove_dir_all(&harness.root).ok();
}

/// A one-second head budget so the 16-byte file splits head/tail.
fn short_head_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            head_seconds: 1,
            chunk_bytes: 4,
            ..base_params()
        },
        ..DeliveryOptions::default()
    }
}
