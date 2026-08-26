use crate::chunk::cancel::cancel_pair;
use crate::chunk::network::{prepare_network, NetworkPreparation};
use crate::debug::network::{NetworkProfile, NetworkThrottle};
use core::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn debug_network_simulation_does_not_add_a_second_connection_queue() {
    let network = NetworkThrottle::new();
    network.update(NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 0,
        packet_loss_bps: 0,
        max_connections_per_host: 1,
    });
    let _occupied = network.acquire("https://relay.example/first");
    let (_handle, token) = cancel_pair();
    let waiting = prepare_network(Some(&network), "https://relay.example/second", &token);
    let prepared = timeout(Duration::from_millis(100), waiting)
        .await
        .expect("the shared request gate owns connection queuing");

    assert!(matches!(prepared, NetworkPreparation::Ready(Some(_))));
    assert_eq!(network.active_connections()[0].1, 2);
}
