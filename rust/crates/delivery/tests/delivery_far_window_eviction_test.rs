//! Storage pressure can evict cached bytes beyond the retrieval window.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::full_disk::{discard, limits, spaced_store};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_with_store;
use std::sync::Arc;
use tokio::time::timeout;

const UNREACHABLE: &str = "http://127.0.0.1:9/video.mp4";

#[tokio::test]
async fn full_store_considers_a_cached_victim_beyond_the_retrieval_window() {
    let fixture = spaced_store("ghostr-far-window-eviction", limits(100, 0), 10_000);
    let items: Vec<_> = (0..27)
        .map(|index| {
            sized_item(
                Box::leak(format!("p{index}").into_boxed_str()),
                UNREACHABLE,
                100,
                1_000,
            )
        })
        .collect();
    seed_range(&fixture.store, &items[0], 0, &[1; 45]).await;
    seed_range(&fixture.store, &items[25], 0, &[8; 55]).await;
    let root = fixture.root.clone();
    let harness = start_harness_with_store(
        Arc::new(fixture.store),
        root.clone(),
        DeliveryOptions::default(),
    );

    harness.handle.update_focus(focus_now(items, 0, 0));
    wait_until_used(&harness, 99).await;

    assert_eq!(
        harness
            .store
            .present_ranges("p0")
            .await
            .expect("valid test fixture"),
        vec![0..45]
    );
    assert_eq!(
        harness
            .store
            .present_ranges("p25")
            .await
            .expect("valid test fixture"),
        vec![0..54]
    );
    assert!(harness.handle.plan_history().iter().all(|evidence| {
        evidence
            .plan
            .allocations
            .iter()
            .all(|allocation| allocation.post.as_str() != "p25")
    }));
    discard(&root);
}

async fn wait_until_used(harness: &delivery_fixture::DeliveryHarness, expected: u64) {
    let result = timeout(Duration::from_secs(30), async {
        while harness.store.used_bytes().await != expected {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    assert!(
        result.is_ok(),
        "far-window eviction: used={}, plans={:?}",
        harness.store.used_bytes().await,
        harness.handle.plan_history()
    );
}
