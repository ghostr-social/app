use crate::api::runtime::discovery::{DiscoveryBoot, DiscoveryRuntime};
use crate::engine::adaptive::DiscoveryDemand;
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::watch;

#[tokio::test]
async fn discovery_boot_uses_only_the_configured_read_and_search_relays() {
    let (_demand_sender, demand) = watch::channel(DiscoveryDemand::Hold);
    let client = Arc::new(Client::default());
    let runtime = DiscoveryRuntime::start(DiscoveryBoot {
        client: std::sync::Arc::clone(&client),
        demand,
        bootstrap: vec!["wss://read.example".to_owned()],
        search_relays: vec!["wss://search.example".to_owned()],
        candidates: None,
    })
    .await;

    assert_eq!(
        runtime.outbox.read().await.relays_for_authors(&[], 12),
        vec!["wss://read.example"]
    );
    assert_eq!(
        runtime.executor.search_relays(),
        vec!["wss://search.example"]
    );
    for url in ["wss://read.example", "wss://search.example"] {
        let relay = client.relay(url).await.expect("configured relay");
        assert!(relay.flags().has_read());
        assert!(!relay.flags().has_write());
    }
}
