use crate::manager::resource_control::{ResourceControl, ResourceEnvironment};
use ghostr_engine::adaptive::ResourceObservation;
use std::time::Duration;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn feedback_closes_old_environment_then_atomically_installs_the_new_one() {
    let old = ResourceEnvironment::new(10, ResourceObservation::new(100, 20, 5, 1));
    let new = ResourceEnvironment::new(30, ResourceObservation::new(200, 40, 7, 2));
    let control = ResourceControl::new(Instant::now(), old);
    control.record_network_bytes(50);

    tokio::time::advance(Duration::from_millis(600)).await;
    let closed = control.feedback(new);
    assert_eq!(closed.actual.storage, 10);
    assert_eq!(closed.target, old.target());

    control.record_network_bytes(100);
    tokio::time::advance(Duration::from_millis(400)).await;
    let current = control.feedback(new);
    assert_eq!(current.actual.storage, 30);
    assert_eq!(current.target, new.target());
    assert_eq!(current.actual.network, 200);
}
