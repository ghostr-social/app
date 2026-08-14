mod gateway_fixture;

use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use ghostr_delivery::playback_demand::{ConsumerId, DemandLease, DemandState};
use ghostr_engine::{ByteRange, PostId};
use std::time::Duration;

#[tokio::test]
async fn progressive_journey_records_gateway_demand() {
    let harness = ProgressiveDeliveryHarness::start("ghostr-progressive-demand-trace");
    let expected = DemandState::Blocked(DemandLease::new(
        ConsumerId::new(1).unwrap(),
        PostId::new("delivery-current"),
        None,
        ByteRange::new(0, 2_048),
    ));

    harness.delivery.demand.emit(expected.clone());
    tokio::time::timeout(Duration::from_secs(1), async {
        while harness.delivery.demands().is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("recorded gateway demand");

    assert_eq!(harness.delivery.demands(), vec![expected]);
}
