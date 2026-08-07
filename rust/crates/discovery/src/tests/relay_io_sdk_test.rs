//! The SDK adapter drains completed reads and contextualizes send failures.

use super::relay_io_relay_fixture::relay_serving;
use crate::relay_io::{RelayBroadcastIo, RelayIo, RelayReadIo, SdkRelayIo};
use nostr_sdk::{Client, EventBuilder, Filter, Keys, Kind};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn cold_disconnected_read_is_not_authoritative_empty() {
    let client = Arc::new(Client::default());
    let relay = "ws://127.0.0.1:1";
    client
        .add_read_relay(relay)
        .await
        .expect("valid local relay URL");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::ZERO);

    let error = io
        .read(RelayReadIo {
            relays: vec![relay.to_owned()],
            filter: Filter::new(),
            timeout: Duration::ZERO,
            progress: None,
        })
        .await
        .expect_err("a cold relay cannot prove that the query is empty");

    assert!(error.to_string().contains("no target relay connected"));
}

#[tokio::test]
async fn rejected_broadcast_carries_adapter_context() {
    let io = SdkRelayIo::new(Arc::new(Client::default()));
    let event = EventBuilder::new(Kind::TextNote, "hello")
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let error = io
        .broadcast(RelayBroadcastIo {
            relays: vec!["not a relay".to_owned()],
            event,
        })
        .await
        .expect_err("the SDK rejects an invalid URL");

    assert!(error.to_string().contains("broadcast failed"));
}

#[tokio::test]
async fn connected_relay_requests_complete_through_the_adapter() {
    let event = EventBuilder::new(Kind::TextNote, "hello")
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let relay = relay_serving(event.clone()).await;
    let client = Arc::new(Client::default());
    client
        .add_relay(&relay)
        .await
        .expect("mock relay should be accepted");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(2));

    io.read(RelayReadIo {
        relays: vec![relay],
        filter: Filter::new(),
        timeout: Duration::from_secs(1),
        progress: None,
    })
    .await
    .expect("connected relay read");

    let broadcast_relay = relay_serving(event.clone()).await;
    let broadcast_client = Arc::new(Client::default());
    broadcast_client
        .add_relay(&broadcast_relay)
        .await
        .expect("mock relay should be accepted");
    let broadcast_io = SdkRelayIo::with_readiness_timeout(broadcast_client, Duration::from_secs(2));
    broadcast_io
        .broadcast(RelayBroadcastIo {
            relays: vec![broadcast_relay],
            event,
        })
        .await
        .expect("connected relay broadcast");
}
