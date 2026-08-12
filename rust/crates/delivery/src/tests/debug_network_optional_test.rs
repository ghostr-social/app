use crate::chunk::cancel::cancel_pair;
use crate::chunk::network::{prepare_network, NetworkPreparation};

#[tokio::test]
async fn absent_debug_throttle_needs_no_network_permit() {
    let (_handle, token) = cancel_pair();

    let prepared = prepare_network(None, "https://relay.example/video", &token).await;

    assert!(matches!(prepared, NetworkPreparation::Ready(None)));
}
