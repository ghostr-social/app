//! Promotion must fetch each planned byte exactly once even when disjoint
//! ranges complete in either order.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, serve_recording, HitLog};
use delivery_fixture::{options::DeliveryOptions, start_harness};
use ghostr_delivery::{debug::network::NetworkProfile, playback_demand::DemandSignal};
use ghostr_engine::{ByteRange, EngineParams, PostId};
use std::time::Duration;

const PLAYBACK_SLICE: u64 = 256 * 1_024;
const TOTAL: u64 = 370_912;

#[tokio::test]
async fn focus_promotion_fetches_exact_disjoint_ranges() {
    let log = hit_log();
    let current = serve_recording("current", vec![1; TOTAL as usize], log.clone()).await;
    let next = serve_recording("next", vec![2; TOTAL as usize], log.clone()).await;
    let harness = start_harness("ghostr-seed-promotion", production_options());
    configure_network(&harness);
    let items = vec![
        sized_item("current", &current, TOTAL, 3_000),
        sized_item("next", &next, TOTAL, 3_000),
    ];
    harness.handle.update_focus(focus_now(items.clone(), 0, 0));
    emit_demand(&harness, "current");
    wait_for_next_gets(&log, 1).await;

    harness.handle.update_focus(focus_now(items, 1, 0));
    emit_demand(&harness, "next");
    wait_for_next_gets(&log, 2).await;

    let mut gets = next_gets(&log);
    gets.sort();
    assert_eq!(gets, ["next:GET:0-262143", "next:GET:262144-370911"]);
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn configure_network(harness: &delivery_fixture::DeliveryHarness) {
    harness.network.update(NetworkProfile {
        bandwidth_kbps: 3_000,
        latency_ms: 450,
        packet_loss_bps: 0,
        max_connections_per_host: 3,
    });
}

fn emit_demand(harness: &delivery_fixture::DeliveryHarness, post: &str) {
    harness.demand.emit(DemandSignal {
        post: PostId::new(post),
        range: ByteRange::new(0, PLAYBACK_SLICE),
    });
}

async fn wait_for_next_gets(log: &HitLog, count: usize) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while next_gets(log).len() < count {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("wanted {count} next GETs, saw {:?}", hits(log)));
}

fn next_gets(log: &HitLog) -> Vec<String> {
    hits(log)
        .into_iter()
        .filter(|hit| hit.starts_with("next:GET:"))
        .collect()
}

fn production_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: PLAYBACK_SLICE,
            ..EngineParams::default()
        },
        ..DeliveryOptions::default()
    }
}
