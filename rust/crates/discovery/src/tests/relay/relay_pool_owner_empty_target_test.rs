//! An empty explicit/configured target never falls through to the SDK pool.

use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::test_support::{read_request, TestRelayIo};
use nostr_sdk::Client;
use std::sync::Arc;

const UNRELATED: &str = "wss://unrelated.example";

#[tokio::test]
async fn empty_configured_target_does_not_query_an_unrelated_pool_relay() {
    let client = Arc::new(Client::default());
    client.add_relay(UNRELATED).await.expect("unrelated relay");
    let io = Arc::new(TestRelayIo::blocked());
    let owner = RelayPoolOwner::with_io(client, RelayPoolConfiguration::default(), io.clone());
    let mut request = read_request(UNRELATED);
    request.relays = None;

    assert!(owner.read(request).await.is_err());
    assert_eq!(io.read_count(), 0);
}
