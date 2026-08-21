use ghostr_delivery::debug::network::{NetworkProfile, NetworkThrottle};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn throttle_uses_scheme_host_and_effective_port() {
    let throttle = NetworkThrottle::new();
    throttle.update(NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 0,
        packet_loss_bps: 0,
        max_connections_per_host: 1,
    });
    let first = throttle.acquire("https://EXAMPLE.com:443/a").await;

    let equivalent = throttle.acquire("https://example.com/b");
    assert!(timeout(Duration::from_millis(25), equivalent)
        .await
        .is_err());
    let other_scheme = throttle.acquire("http://example.com/a");
    assert!(timeout(Duration::from_millis(100), other_scheme)
        .await
        .is_ok());

    drop(first);
    let released = throttle.acquire("https://example.com/c");
    assert!(timeout(Duration::from_millis(100), released).await.is_ok());
}
