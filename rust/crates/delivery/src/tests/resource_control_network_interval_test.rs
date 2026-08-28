use crate::manager::resource_control::{ResourceControl, ResourceEnvironment};
use core::time::Duration;
use ghostr_engine::adaptive::ResourceObservation;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn network_bytes_keep_the_producer_interval_when_planning_is_delayed() {
    let environment = environment();
    let control = ResourceControl::new(Instant::now(), environment);

    tokio::time::advance(Duration::from_millis(100)).await;
    control.record_network_bytes(100_000);
    tokio::time::advance(Duration::from_millis(400)).await;
    let first = control.feedback(environment);
    assert_eq!(first.actual.network, 200_000);
    assert_eq!(
        first
            .price_snapshot
            .expect("valid test fixture")
            .cursor
            .revision,
        1
    );

    tokio::time::advance(Duration::from_millis(100)).await;
    control.record_network_bytes(200_000);
    tokio::time::advance(Duration::from_millis(400)).await;
    let second = control.feedback(environment);
    assert_eq!(second.actual.network, 400_000);
    assert_eq!(
        second
            .price_snapshot
            .expect("valid test fixture")
            .cursor
            .revision,
        2
    );
}

#[tokio::test(start_paused = true)]
async fn a_late_burst_never_backfills_the_already_closed_interval() {
    let environment = environment();
    let control = ResourceControl::new(Instant::now(), environment);

    tokio::time::advance(Duration::from_millis(600)).await;
    control.record_network_bytes(100_000);
    let closed = control.feedback(environment);
    assert_eq!(closed.actual.network, 0);
    assert_eq!(
        closed
            .price_snapshot
            .expect("valid test fixture")
            .cursor
            .revision,
        1
    );

    tokio::time::advance(Duration::from_millis(400)).await;
    let next = control.feedback(environment);
    assert_eq!(next.actual.network, 200_000);
    assert_eq!(
        next.price_snapshot
            .expect("valid test fixture")
            .cursor
            .revision,
        2
    );
}

fn environment() -> ResourceEnvironment {
    ResourceEnvironment::new(0, ResourceObservation::new(100_000, 100, 10, 2))
}
