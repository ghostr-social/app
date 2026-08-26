//! A route snapshot and its network operation share one configuration lease.

use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::session_generation::SessionGeneration;
use crate::test_support::{read_request, TestRelayIo};
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::oneshot;

const OLD: &str = "wss://old.example";
const NEW: &str = "wss://new.example";

#[tokio::test]
async fn queued_configuration_cannot_replace_a_resolved_route_before_read() {
    let client = Arc::new(Client::default());
    let io = Arc::new(TestRelayIo::blocked());
    let owner = Arc::new(RelayPoolOwner::with_io(
        std::sync::Arc::clone(&client),
        configuration(OLD),
        std::sync::Arc::<TestRelayIo>::clone(&io),
    ));
    let route = owner
        .begin_route(SessionGeneration::initial())
        .await
        .expect("route lease");
    let config_owner = std::sync::Arc::clone(&owner);
    let (finished, mut config_done) = oneshot::channel();
    tokio::spawn(async move {
        let mut guard = config_owner.begin_configuration().await;
        guard.replace_configuration(configuration(NEW)).await;
        let _ = finished.send(());
    });
    tokio::task::yield_now().await;

    let read = tokio::spawn({
        let route = std::sync::Arc::clone(&route);
        async move { route.read(read_request(OLD)).await }
    });
    io.query_started.notified().await;
    assert!(config_done.try_recv().is_err());
    io.release_query();
    assert!(read.await.expect("read task").is_ok());
    assert!(config_done.try_recv().is_err());
    drop(route);
    config_done.await.expect("configuration completes");

    assert!(client.relay(OLD).await.is_err());
    assert!(client.relay(NEW).await.is_ok());
}

fn configuration(relay: &str) -> RelayPoolConfiguration {
    RelayPoolConfiguration {
        read_relays: vec![relay.to_owned()],
        search_relays: Vec::new(),
    }
}
