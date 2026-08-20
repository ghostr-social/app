//! A real manager treats a finalized cached video as one eviction unit.

mod delivery_fixture;

use delivery_fixture::full_disk::{discard, limits, spaced_store};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_with_store;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

const UNREACHABLE: &str = "http://127.0.0.1:9/video.mp4";

#[tokio::test]
async fn storage_pressure_evicts_a_far_finalized_video_atomically() {
    let fixture = spaced_store("ghostr-finalized-manager-eviction", limits(100, 0), 10_000);
    let items: Vec<_> = (0..9)
        .map(|index| {
            let size = if index == 8 { 55 } else { 100 };
            sized_item(
                Box::leak(format!("p{index}").into_boxed_str()),
                UNREACHABLE,
                size,
                1_000,
            )
        })
        .collect();
    seed_range(&fixture.store, &items[1], 0, &[1; 45]).await;
    seed_range(&fixture.store, &items[8], 0, &[8; 55]).await;
    fixture.store.finalize("p8", None).await.unwrap();
    let root = fixture.root.clone();
    let harness = start_harness_with_store(
        Arc::new(fixture.store),
        root.clone(),
        DeliveryOptions::default(),
    );

    harness.handle.update_focus(focus_now(items, 1, 0));
    wait_until_evicted(&harness).await;

    assert!(
        planned_whole_eviction(&harness.handle),
        "finalized eviction plans: {:#?}",
        harness.handle.plan_history()
    );
    assert_eq!(
        harness.store.present_ranges("p1").await.unwrap(),
        vec![0..45]
    );
    assert!(harness.store.present_ranges("p8").await.unwrap().is_empty());
    discard(&root);
}

fn planned_whole_eviction(handle: &ghostr_delivery::delivery_events::DeliveryHandle) -> bool {
    handle.plan_history().iter().any(|evidence| {
        evidence.plan.evictions.iter().any(|eviction| {
            eviction.post.as_str() == "p8" && eviction.range == ghostr_engine::ByteRange::new(0, 55)
        })
    })
}

async fn wait_until_evicted(harness: &delivery_fixture::DeliveryHarness) {
    timeout(Duration::from_secs(2), async {
        loop {
            if harness.store.used_bytes().await == 45 && planned_whole_eviction(&harness.handle) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("finalized eviction");
}
