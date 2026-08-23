use crate::relay::pool::RelayReadRequest;
use crate::test_support::read_request;
use crate::tests::relay_health_owner_fixture::{health_owner, RegistrationLog};
use crate::tests::relay_io_health_fixture::HealthRelayIo;
use std::sync::Arc;
use std::time::Duration;

const HEALTHY: &str = "wss://healthy.example";
const FAILING: &str = "wss://failing.example";

#[tokio::test(start_paused = true)]
async fn owner_skips_cooling_relay_then_reintroduces_one_recovery_probe() {
    let io = Arc::new(HealthRelayIo::new(FAILING));
    let registrations = Arc::new(RegistrationLog::default());
    let owner = health_owner(io.clone(), registrations.clone());
    assert!(
        !owner
            .read(request([HEALTHY, FAILING]))
            .await
            .expect("partial")
            .complete
    );
    assert!(
        !owner
            .read(request([HEALTHY, FAILING]))
            .await
            .expect("narrowed retry")
            .complete
    );
    assert_eq!(
        io.reads(),
        vec![relays([HEALTHY, FAILING]), relays([HEALTHY])]
    );
    assert_eq!(registrations.count(FAILING), 1);

    tokio::time::advance(Duration::from_secs(2)).await;
    io.recover();
    assert!(
        owner
            .read(request([HEALTHY, FAILING]))
            .await
            .expect("recovery probe")
            .complete
    );
    assert_eq!(registrations.count(FAILING), 2);
}

#[tokio::test(start_paused = true)]
async fn all_cooling_targets_return_retryable_incomplete_without_io() {
    let io = Arc::new(HealthRelayIo::new(FAILING));
    let owner = health_owner(io.clone(), Arc::new(RegistrationLog::default()));
    assert!(
        !owner
            .read(request([FAILING]))
            .await
            .expect("failure")
            .complete
    );
    let retry = owner
        .read(request([FAILING]))
        .await
        .expect("cooldown retry");

    assert!(!retry.complete);
    assert!(retry.events.is_empty());
    assert_eq!(io.reads(), vec![relays([FAILING])]);
}

fn relays<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_owned).collect()
}

fn request<const N: usize>(values: [&str; N]) -> RelayReadRequest {
    let mut request = read_request(values[0]);
    request.relays = Some(relays(values));
    request
}
