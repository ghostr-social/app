//! The SDK adapter drains completed reads and contextualizes send failures.

use crate::discovery::relay_io::{RelayBroadcastIo, RelayIo, RelayReadIo, SdkRelayIo};
use nostr_sdk::{Client, EventBuilder, Filter, Keys, Kind};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn configured_zero_timeout_read_drains_to_empty() {
    let client = Arc::new(Client::default());
    let relay = "ws://127.0.0.1:1";
    client
        .add_read_relay(relay)
        .await
        .expect("valid local relay URL");
    let io = SdkRelayIo::new(client);

    let events = io
        .read(RelayReadIo {
            relays: vec![relay.to_owned()],
            filter: Filter::new(),
            timeout: Duration::ZERO,
            progress: None,
        })
        .await
        .expect("the stream itself is valid");

    assert!(events.is_empty());
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
