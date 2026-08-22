use ghostr_delivery::debug::network::{NetworkProfile, NetworkThrottle};

#[tokio::test]
async fn connection_accounting_uses_scheme_host_and_effective_port() {
    let throttle = NetworkThrottle::new();
    throttle.update(NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 0,
        packet_loss_bps: 0,
        max_connections_per_host: 0,
    });
    let first = throttle.acquire("https://EXAMPLE.com:443/a").await;
    let equivalent = throttle.acquire("https://example.com/b").await;
    let other_scheme = throttle.acquire("http://example.com/a").await;

    assert_eq!(
        throttle.active_connections(),
        vec![
            ("http://example.com".to_owned(), 1),
            ("https://example.com".to_owned(), 2),
        ]
    );

    drop(other_scheme);
    drop(equivalent);
    drop(first);
    assert!(throttle.active_connections().is_empty());
}
