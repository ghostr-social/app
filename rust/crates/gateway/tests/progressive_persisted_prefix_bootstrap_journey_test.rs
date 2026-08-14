mod gateway_fixture;

use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use gateway_fixture::progressive_journey_item::unknown_item;
use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::catalog::Catalog;
use std::time::Duration;

const STORED_PREFIX: usize = 262_144;

#[tokio::test]
async fn range_opaque_head_advances_past_a_persisted_bootstrap_prefix() {
    let origin = ProgressiveJourneyOrigin::with_range_opaque_head().await;
    let harness = ProgressiveDeliveryHarness::start("ghostr-persisted-prefix-bootstrap");
    let mut item = unknown_item("delivery-current", &origin.url);
    item.meta.size_bytes = Some(origin.total_bytes());
    item.meta.duration_ms = Some(10_000);
    seed_prefix(&harness, &item).await;

    harness.focus(vec![item], 0);

    let outcome = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if origin
                .get_ranges()
                .iter()
                .any(|range| range.start >= STORED_PREFIX as u64)
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    assert!(
        outcome.is_ok(),
        "requests={:?}, plans={:#?}",
        origin.requests(),
        harness.delivery.handle.plan_history(),
    );
}

async fn seed_prefix(harness: &ProgressiveDeliveryHarness, item: &FocusItem) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(item.post.clone(), item.meta.clone());
    let identity = binding.transfer(&item.meta.urls[0]).expect("identity");
    harness
        .delivery
        .store
        .bind_representation(binding)
        .await
        .unwrap();
    harness.delivery.store.select_transfer(identity.clone());
    assert!(harness
        .delivery
        .store
        .write_range_for_transfer_if_current(&identity, 0, &vec![1; STORED_PREFIX])
        .await
        .unwrap());
}
