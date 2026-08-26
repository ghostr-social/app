use core::time::Duration;
use ghostr_delivery::debug::network::{NetworkProfile, NetworkThrottle};
use tokio::time::Instant;

#[test]
fn per_host_setting_does_not_create_a_second_admission_queue() {
    let throttle = NetworkThrottle::new();
    throttle.update(NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 0,
        packet_loss_bps: 0,
        max_connections_per_host: 1,
    });
    let first = throttle.acquire("https://relay.example/a");
    let second = throttle.acquire("https://relay.example/b");

    assert_eq!(
        throttle.active_connections(),
        vec![("https://relay.example".to_owned(), 2)]
    );
    drop(second);
    drop(first);
    assert!(throttle.active_connections().is_empty());
}

#[tokio::test(start_paused = true)]
async fn latency_and_bandwidth_are_applied_to_transfer_time() {
    let throttle = NetworkThrottle::new();
    throttle.update(NetworkProfile {
        bandwidth_kbps: 8,
        latency_ms: 250,
        packet_loss_bps: 0,
        max_connections_per_host: 0,
    });
    let started = Instant::now();

    throttle.wait_for_latency().await;
    throttle.pace(1_000).await;

    assert!(started.elapsed() >= Duration::from_millis(1_250));
}
