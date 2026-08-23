//! An early relay-wide AUTH failure must settle an exact auth-required query.

use crate::relay::io::{RelayIo, RelayReadIo, SdkRelayIo};
use crate::tests::relay_io_auth_fixture::{
    auth_closed_then_stale_eose_relay, auth_failure_before_closed_relay,
};
use crate::tests::relay_io_auth_retry_fixture::auth_retry_relay;
use nostr_sdk::{Client, EventBuilder, Filter, Keys};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn auth_failure_before_exact_closed_does_not_wait_for_query_timeout() {
    let relay = auth_failure_before_closed_relay().await;
    let client = Arc::new(Client::default());
    client.add_relay(&relay).await.expect("mock relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));
    let read = io.read(RelayReadIo {
        relays: vec![relay],
        filter: Filter::new(),
        timeout: Duration::from_secs(2),
        progress: None,
        admissions: None,
    });

    let result = tokio::time::timeout(Duration::from_millis(500), read)
        .await
        .expect("authentication failure should settle immediately");
    let error = result.expect_err("authentication must fail");
    let message = format!("{error:#}");
    assert!(message.contains("authentication failed"), "{message}");
}

#[tokio::test]
async fn stale_eose_cannot_complete_an_auth_blocked_subscription() {
    let relay = auth_closed_then_stale_eose_relay().await;
    let client = Arc::new(Client::default());
    client.add_relay(&relay).await.expect("mock relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));
    let read = io.read(RelayReadIo {
        relays: vec![relay],
        filter: Filter::new(),
        timeout: Duration::from_millis(100),
        progress: None,
        admissions: None,
    });

    let result = tokio::time::timeout(Duration::from_millis(500), read)
        .await
        .expect("blocked subscription settles at its deadline");

    assert!(
        result.is_err(),
        "pre-authentication EOSE is not authoritative"
    );
}

#[tokio::test]
async fn authenticated_retry_uses_a_fresh_subscription_identity() {
    let event = EventBuilder::text_note("authenticated")
        .sign_with_keys(&Keys::generate())
        .unwrap();
    let relay = auth_retry_relay(event.clone()).await;
    let client = Arc::new(Client::builder().signer(Keys::generate()).build());
    client.add_relay(&relay).await.expect("mock relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));

    let result = io
        .read(RelayReadIo {
            relays: vec![relay],
            filter: Filter::new(),
            timeout: Duration::from_secs(1),
            progress: None,
            admissions: None,
        })
        .await
        .expect("authenticated retry");

    assert_eq!(result.events, vec![event]);
    assert!(result.complete);
}
