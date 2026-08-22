use super::items::{focus_now, sized_item};
use super::options::{base_params, DeliveryOptions};
use super::{start_harness, DeliveryHarness};
use ghostr_delivery::playback_demand::DemandConsumer;
use ghostr_engine::{ByteRange, EngineParams};
use std::num::NonZeroUsize;
use std::time::Duration;

pub const POSTS: [&str; 4] = ["current", "next-1", "next-2", "next-3"];

pub async fn start(url: &str) -> (DeliveryHarness, DemandConsumer) {
    let harness = start_harness("ghostr-protected-capacity-trial", options());
    harness.handle.update_focus(focus_now(items(url), 0, 0));
    let demand = super::demand::blocked(&harness, "current", ByteRange::new(0, 8)).await;
    (harness, demand)
}

pub async fn wait_for_bytes(harness: &DeliveryHarness, expected: u64) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if stored_bytes(harness).await >= expected {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("both active requests reach the store");
}

async fn stored_bytes(harness: &DeliveryHarness) -> u64 {
    let mut stored = 0;
    for post in POSTS {
        stored += harness
            .store
            .present_ranges(post)
            .await
            .unwrap()
            .iter()
            .map(|range| range.end - range.start)
            .sum::<u64>();
    }
    stored
}

fn items(url: &str) -> Vec<ghostr_delivery::delivery_events::FocusItem> {
    let mut items = vec![sized_item("current", url, 8, 1_000)];
    items.extend(POSTS[1..].iter().map(|id| sized_item(id, url, 32, 4_000)));
    items
}

fn options() -> DeliveryOptions {
    let mut options = DeliveryOptions {
        params: EngineParams {
            balanced_concurrency: 3,
            ..base_params()
        },
        ..DeliveryOptions::default()
    };
    options.tuning.max_requests_per_authority = NonZeroUsize::new(3);
    options
}
