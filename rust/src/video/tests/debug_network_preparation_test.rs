use crate::video::chunk_cancel::cancel_pair;
use crate::video::chunk_network::{prepare_network, NetworkPreparation};
use crate::video::debug_network::{NetworkProfile, NetworkThrottle};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn cancelled_transfer_stops_waiting_for_a_host_slot() {
    let network = NetworkThrottle::new();
    network.update(NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 0,
        max_connections_per_host: 1,
    });
    let _occupied = network.acquire("https://relay.example/first").await;
    let (handle, token) = cancel_pair();
    let waiting = prepare_network(Some(&network), "https://relay.example/second", &token);
    tokio::pin!(waiting);
    assert!(timeout(Duration::from_millis(10), waiting.as_mut())
        .await
        .is_err());

    handle.cancel();

    let prepared = timeout(Duration::from_millis(100), waiting)
        .await
        .expect("cancelled wait should finish");
    assert!(matches!(prepared, NetworkPreparation::Cancelled));
}
