//! High RTT gives every protected post one slice before any depth.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, serve_recording};
use delivery_fixture::{options::DeliveryOptions, start_harness, wait::wait_for_ranges};
use ghostr_delivery::{debug::network::NetworkProfile, playback_demand::DemandSignal};
use ghostr_engine::{ByteRange, EngineParams, PostId};
use std::time::Duration;
use tokio::time::Instant;

const MINIMUM: u64 = 49_152;
const PLAYBACK_SLICE: u64 = 256 * 1_024;
const TOTAL: u64 = 370_912;
const POSTS: [&str; 4] = ["current", "next-1", "next-2", "next-3"];

#[tokio::test]
async fn high_rtt_serves_protected_breadth_before_depth() {
    let log = hit_log();
    let origin = serve_recording("shared", vec![7; TOTAL as usize], log.clone()).await;
    let harness = start_harness("ghostr-startup-breadth", production_options());
    configure_network(&harness);
    let items = POSTS.map(|id| sized_item(id, &origin, TOTAL, 3_000));

    harness.handle.update_focus(focus_now(items.to_vec(), 0, 0));
    emit_playback_demand(&harness, "current");
    let promotion_at = Instant::now() + Duration::from_millis(1_800);
    let warm = warm_monitor(&harness);
    assert_promoted_ready(&harness).await;
    tokio::time::sleep_until(promotion_at).await;
    harness.handle.update_focus(focus_now(items.to_vec(), 1, 0));
    emit_playback_demand(&harness, "next-1");
    await_warm(warm).await;
    assert_origin_order(&log);
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

fn warm_monitor(harness: &delivery_fixture::DeliveryHarness) -> tokio::task::JoinHandle<()> {
    let store = harness.store.clone();
    tokio::spawn(async move {
        for post in POSTS {
            wait_for_ranges(&store, post, &[(0, MINIMUM)]).await;
        }
    })
}

async fn assert_promoted_ready(harness: &delivery_fixture::DeliveryHarness) {
    tokio::time::timeout(
        Duration::from_millis(1_750),
        wait_for_ranges(&harness.store, "next-1", &[(0, MINIMUM)]),
    )
    .await
    .expect("promoted post is ready before focus changes");
}

async fn await_warm(warm: tokio::task::JoinHandle<()>) {
    tokio::time::timeout(Duration::from_secs(4), warm)
        .await
        .expect("protected prefix reaches its bootstrap milestone")
        .expect("warm monitor remains live");
}

fn assert_origin_order(log: &delivery_fixture::media::HitLog) {
    let gets: Vec<_> = hits(log)
        .into_iter()
        .filter(|hit| hit.contains(":GET:"))
        .take(4)
        .collect();
    let current = "shared:GET:0-370911";
    let slice = "shared:GET:0-262143";
    assert_eq!(gets, [current, slice, slice, slice]);
}

fn emit_playback_demand(harness: &delivery_fixture::DeliveryHarness, post: &str) {
    harness.demand.emit(DemandSignal {
        post: PostId::new(post),
        range: ByteRange::new(0, PLAYBACK_SLICE),
    });
}

fn production_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams::default(),
        ..DeliveryOptions::default()
    }
}
