//! Session reset drops account-specific relay routes from the shared client.

use crate::api::feed_runtime::{DiscoveryBoot, DiscoveryRuntime};
use crate::engine::inventory_controller::Mode;
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::watch;

const READ_RELAY: &str = "wss://read.example";
const SEARCH_RELAY: &str = "wss://search.example";
const DYNAMIC_RELAY: &str = "wss://dynamic.example";

#[tokio::test]
async fn reset_keeps_only_current_configured_relays() {
    let client = Arc::new(Client::default());
    for relay in [READ_RELAY, SEARCH_RELAY, DYNAMIC_RELAY] {
        client.add_relay(relay).await.expect("relay");
    }
    let (_modes, mode_updates) = watch::channel(Mode::Comfort);
    let runtime = DiscoveryRuntime::start(DiscoveryBoot {
        client: client.clone(),
        modes: mode_updates,
        bootstrap: vec![READ_RELAY.to_owned()],
        search_relays: vec![SEARCH_RELAY.to_owned()],
        candidates: None,
    })
    .await;
    client
        .add_relay(DYNAMIC_RELAY)
        .await
        .expect("dynamic relay");

    runtime.reset_session(None).await;

    assert!(client.relay(READ_RELAY).await.is_ok());
    assert!(client.relay(SEARCH_RELAY).await.is_ok());
    assert!(client.relay(DYNAMIC_RELAY).await.is_err());
}
