use crate::api::feed_runtime::{DiscoveryBoot, DiscoveryRuntime};
use crate::engine::inventory_controller::Mode;
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::watch;

#[tokio::test]
async fn discovery_boot_uses_only_the_configured_read_and_search_relays() {
    let (_modes, mode_updates) = watch::channel(Mode::Comfort);
    let client = Arc::new(Client::default());
    let runtime = DiscoveryRuntime::start(DiscoveryBoot {
        client: client.clone(),
        modes: mode_updates,
        bootstrap: vec!["wss://read.example".to_owned()],
        search_relays: vec!["wss://search.example".to_owned()],
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
