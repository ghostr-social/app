use crate::manager::resource_control::{ResourceControl, ResourceEnvironment};
use ghostr_engine::adaptive::ResourceObservation;
use core::time::Duration;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn cpu_and_request_costs_share_the_exact_completion_interval() {
    let environment = ResourceEnvironment::new(0, ResourceObservation::new(1, 100, 10, 2));
    let control = ResourceControl::new(Instant::now(), environment);
    tokio::time::advance(Duration::from_millis(499)).await;
    control.record_cpu_ms(7);
    control.record_cpu_ms(5);
    control.record_request();
    control.record_request();

    tokio::time::advance(Duration::from_millis(1)).await;
    let feedback = control.feedback(environment);
    assert_eq!(feedback.actual.cpu, 12);
    assert_eq!(feedback.actual.requests, 2);
    assert_eq!(feedback.target.cpu, 10);
    assert_eq!(feedback.target.requests, 2);
}

#[tokio::test(start_paused = true)]
async fn boundary_race_always_places_cpu_in_the_new_half_open_interval() {
    let environment = ResourceEnvironment::new(0, ResourceObservation::new(1, 100, 10, 2));
    let control = ResourceControl::new(Instant::now(), environment);
    tokio::time::advance(Duration::from_millis(500)).await;
    let recorder = control.clone();
    let sampler = control.clone();
    let record = tokio::spawn(async move {
        recorder.record_cpu_ms(7);
        recorder.record_request();
    });
    let sample = tokio::spawn(async move { sampler.feedback(environment) });
    let (_, first) = tokio::join!(record, sample);

    let first = first.expect("valid test fixture");
    assert_eq!(first.price_snapshot.expect("valid test fixture").cursor.revision, 1);
    assert_eq!(first.actual.cpu, 0);
    assert_eq!(first.actual.requests, 0);
    tokio::time::advance(Duration::from_millis(500)).await;
    let second = control.feedback(environment);
    assert_eq!(second.price_snapshot.expect("valid test fixture").cursor.revision, 2);
    assert_eq!(second.actual.cpu, 7);
    assert_eq!(second.actual.requests, 1);
}
