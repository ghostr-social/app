//! Store progress may replan priorities, but one paid-RTT protected grant
//! remains the bounded scheduling quantum until its range completes.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, serve_recording, HitLog};
use delivery_fixture::{
    options::DeliveryOptions,
    start_harness,
    wait::{wait_for_ranges, wait_until},
};
use ghostr_delivery::{debug::network::NetworkProfile, playback_demand::DemandSignal};
use ghostr_engine::{ByteRange, EngineParams, PostId};
use std::time::Duration;

const FLOOR: u64 = 49_152;
const PLAYBACK_SLICE: u64 = 256 * 1_024;
const TOTAL: u64 = 370_912;
const POSTS: [&str; 4] = ["current", "next-1", "next-2", "next-3"];

#[tokio::test]
async fn high_rtt_subwrites_finish_each_bounded_seed_without_shifted_restarts() {
    let log = hit_log();
    let origin = serve_recording("shared", vec![7; TOTAL as usize], log.clone()).await;
    let harness = start_harness("ghostr-seed-stability", production_options());
    configure_network(&harness);
    let items = POSTS.map(|id| sized_item(id, &origin, TOTAL, 3_000));
    harness.handle.update_focus(focus_now(items.to_vec(), 0, 0));
    emit_demand(&harness);

    tokio::time::timeout(Duration::from_secs(4), async {
        wait_for_partial_seed(&harness).await;
        harness.handle.update_focus(focus_now(items.to_vec(), 0, 0));
        warm_all(&harness).await;
        wait_for_ranges(&harness.store, "next-1", &[(0, 64 * 1_024 + 1)]).await;
    })
    .await
    .expect("protected startup floors before the live deadline");
    assert_no_shifted_seed_restarts(&log);

    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn configure_network(harness: &delivery_fixture::DeliveryHarness) {
    harness.network.update(NetworkProfile {
        bandwidth_kbps: 2_500,
        latency_ms: 450,
        max_connections_per_host: 3,
    });
}

fn emit_demand(harness: &delivery_fixture::DeliveryHarness) {
    harness.demand.emit(DemandSignal {
        post: PostId::new("current"),
        range: ByteRange::new(0, PLAYBACK_SLICE),
    });
}

async fn warm_all(harness: &delivery_fixture::DeliveryHarness) {
    for post in POSTS {
        wait_for_ranges(&harness.store, post, &[(0, FLOOR)]).await;
    }
}

async fn wait_for_partial_seed(harness: &delivery_fixture::DeliveryHarness) {
    wait_until(&harness.store, "next-1", |ranges| {
        ranges
            .iter()
            .any(|range| range.start == 0 && range.end >= 16 * 1_024 && range.end < PLAYBACK_SLICE)
    })
    .await;
}

fn assert_no_shifted_seed_restarts(log: &HitLog) {
    let shifted: Vec<_> = hits(log)
        .into_iter()
        .filter(|hit| is_mid_grant_start(hit))
        .collect();
    assert!(shifted.is_empty(), "shifted requests: {shifted:?}");
}

fn is_mid_grant_start(hit: &str) -> bool {
    let Some(range) = hit.strip_prefix("shared:GET:") else {
        return false;
    };
    let start = range.split('-').next().unwrap_or(range);
    start
        .parse::<u64>()
        .is_ok_and(|value| value > 0 && value < PLAYBACK_SLICE)
}

fn production_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams::default(),
        ..DeliveryOptions::default()
    }
}
