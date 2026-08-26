//! Relay connection readiness does not consume the subscription response budget.

use crate::relay::io::{RelayIo as _, RelayReadIo, SdkRelayIo};
use crate::tests::relay_io_delayed_fixture::delayed_relay;
use core::time::Duration;
use nostr_sdk::{Client, Filter};
use std::sync::Arc;

#[tokio::test]
async fn slow_connection_still_receives_a_full_query_response_budget() {
    let relay = delayed_relay(None, Duration::from_millis(100)).await;
    let client = Arc::new(Client::default());
    client.add_relay(&relay).await.expect("mock relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));

    let result = io
        .read(RelayReadIo {
            relays: vec![relay],
            filter: Filter::new(),
            timeout: Duration::from_millis(50),
            progress: None,
            admissions: None,
        })
        .await
        .expect("EOSE after a slow connection");

    assert!(result.complete);
    assert!(result.events.is_empty());
}
