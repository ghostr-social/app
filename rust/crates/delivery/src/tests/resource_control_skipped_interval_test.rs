use crate::manager::resource_control::{ResourceControl, ResourceEnvironment};
use core::time::Duration;
use ghostr_engine::adaptive::ResourceObservation;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn skipped_empty_intervals_update_prices_exactly_without_iteration() {
    let target = ResourceObservation::new(100_000, 100, 10, 2);
    let environment = ResourceEnvironment::new(150, target);
    let control = ResourceControl::new(Instant::now(), environment);
    control.record_network_bytes(100_000);
    tokio::time::advance(Duration::from_millis(500)).await;
    let first = control.feedback(environment);
    assert_eq!(
        first
            .price_snapshot
            .expect("valid test fixture")
            .prices
            .network_micros,
        1_000
    );

    tokio::time::advance(Duration::from_secs(500000)).await;
    let skipped = control.feedback(environment);
    let snapshot = skipped.price_snapshot.expect("valid test fixture");
    assert_eq!(snapshot.cursor.revision, 1_000_001);
    assert_eq!(snapshot.prices.network_micros, 0);
    assert_eq!(snapshot.prices.storage_micros, 500_000_500);
    assert_eq!(skipped.actual, ResourceObservation::new(0, 150, 0, 0));
}
