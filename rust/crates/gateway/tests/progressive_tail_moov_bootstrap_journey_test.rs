mod gateway_fixture;

use core::time::Duration;
use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use gateway_fixture::progressive_journey_item::unknown_item;
use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;

#[tokio::test]
async fn tail_moov_bootstrap_fetches_metadata_without_refetching_the_prefix() {
    let origin = ProgressiveJourneyOrigin::tail_moov_with_blocked_head().await;
    let harness = ProgressiveDeliveryHarness::start("ghostr-progressive-tail-moov");
    harness.focus(vec![unknown_item("delivery-current", &origin.url)], 0);

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let ranges = harness
                .delivery
                .store
                .present_ranges("delivery-current")
                .await
                .expect("valid test fixture");
            if ranges.iter().any(|range| range.end == origin.total_bytes()) {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("tail metadata range");

    let ranges = origin.get_ranges();
    assert!(ranges.iter().any(|range| range.start == 0));
    assert!(ranges.iter().any(|range| range.end == origin.total_bytes()));
    assert!(ranges.windows(2).all(|pair| pair[0].end <= pair[1].start));
}
