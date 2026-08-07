//! Live configuration makes its relay set authoritative over the client pool.

use crate::api::runtime::discovery::{DiscoveryBoot, DiscoveryRuntime};
use crate::api::runtime::configuration;
use crate::engine::inventory_controller::Mode;
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::watch;

#[tokio::test]
async fn live_configuration_removes_unconfigured_pool_relays() {
    let client = Arc::new(Client::default());
    for relay in [
        "wss://old-config.example",
        "wss://dynamic.example",
        "wss://kept.example",
    ] {
        client.add_relay(relay).await.expect("relay");
    }
    let (_modes, mode_updates) = watch::channel(Mode::Comfort);
    let runtime = DiscoveryRuntime::start(DiscoveryBoot {
        client: client.clone(),
        modes: mode_updates,
        bootstrap: vec!["wss://old-config.example".to_owned()],
        search_relays: vec!["wss://kept.example".to_owned()],
        candidates: None,
    })
    .await;

    let mut transition = runtime.relay_pool.begin_configuration().await;
    configuration::replace_relays(
        &runtime,
        &mut transition,
        vec!["wss://kept.example".to_owned()],
        Vec::new(),
    )
    .await;

    assert!(client.relay("wss://old-config.example").await.is_err());
    assert!(client.relay("wss://dynamic.example").await.is_err());
    assert!(client.relay("wss://kept.example").await.is_ok());
}
