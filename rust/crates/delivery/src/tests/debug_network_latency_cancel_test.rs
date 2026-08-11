use crate::chunk::cancel::cancel_pair;
use crate::chunk::network::{prepare_network, NetworkPreparation};
use crate::debug::network::{NetworkProfile, NetworkThrottle};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn cancelled_transfer_stops_waiting_for_injected_latency() {
    let network = NetworkThrottle::new();
    network.update(NetworkProfile {
        latency_ms: 60_000,
        ..NetworkProfile::default()
    });
    let (handle, token) = cancel_pair();
    let waiting = prepare_network(Some(&network), "https://relay.example/video", &token);
    tokio::pin!(waiting);
    assert!(timeout(Duration::from_millis(10), waiting.as_mut())
        .await
        .is_err());

    handle.cancel();

    let prepared = timeout(Duration::from_millis(100), waiting)
        .await
        .expect("cancelled latency should finish");
    assert!(matches!(prepared, NetworkPreparation::Cancelled));
}
