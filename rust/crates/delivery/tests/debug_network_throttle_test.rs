use ghostr_delivery::debug::network::{NetworkProfile, NetworkThrottle};
use std::time::Duration;
use tokio::time::{timeout, Instant};

#[tokio::test]
async fn per_host_cap_blocks_only_matching_host_connections() {
    let throttle = NetworkThrottle::new();
    throttle.update(NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 0,
        packet_loss_bps: 0,
        max_connections_per_host: 1,
    });
    let first = throttle.acquire("https://relay.example/a").await;
    let same_host = throttle.acquire("https://relay.example/b");

    assert!(timeout(Duration::from_millis(10), same_host).await.is_err());
    let other_host = throttle.acquire("https://mirror.example/a").await;
    drop(other_host);
    drop(first);

    let released = throttle.acquire("https://relay.example/c");
    assert!(timeout(Duration::from_millis(100), released).await.is_ok());
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
