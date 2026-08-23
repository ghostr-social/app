//! Duplicate events do not consume the bound and overflow is not authoritative.

use crate::relay::health::RelayHealth;
use crate::relay::io::{RelayIo, RelayReadIo, SdkRelayIo};
use crate::tests::relay_io_relay_fixture::relay_serving_events;
use nostr_sdk::{Client, Event, EventBuilder, Filter, Keys, Kind, Timestamp};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn unique_event_overflow_returns_a_bounded_partial_answer() {
    let first = signed("first");
    let second = signed("second");
    let excess = signed("excess");
    let relay = relay_serving_events(vec![first.clone(), first, second, excess]).await;
    let client = Arc::new(Client::default());
    client.add_relay(&relay).await.expect("mock relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));
    let health = Arc::new(RelayHealth::new());

    let result = io
        .read(RelayReadIo {
            relays: vec![relay.clone()],
            filter: Filter::new().kind(Kind::Custom(22)).limit(2),
            timeout: Duration::from_secs(1),
            progress: None,
            admissions: Some(health.batch(std::slice::from_ref(&relay))),
        })
        .await
        .expect("safe prefix remains usable");

    assert_eq!(result.events.len(), 2);
    assert!(!result.complete);
    assert_eq!(
        health.batch(std::slice::from_ref(&relay)).urls(),
        vec![relay]
    );
}

#[tokio::test]
async fn filter_limit_applies_to_each_relay_without_truncating_the_union() {
    let expected = [
        signed_at("d", 40),
        signed_at("c", 30),
        signed_at("b", 20),
        signed_at("a", 10),
    ];
    let first = relay_serving_events(vec![expected[3].clone(), expected[0].clone()]).await;
    let second = relay_serving_events(vec![expected[2].clone(), expected[1].clone()]).await;
    let client = Arc::new(Client::default());
    client.add_relay(&first).await.expect("first relay");
    client.add_relay(&second).await.expect("second relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));

    let result = io
        .read(RelayReadIo {
            relays: vec![first, second],
            filter: Filter::new().kind(Kind::Custom(22)).limit(2),
            timeout: Duration::from_secs(1),
            progress: None,
            admissions: None,
        })
        .await
        .expect("bounded union remains usable");

    assert_eq!(result.events, expected);
    assert!(result.complete);
}

fn signed(content: &str) -> Event {
    EventBuilder::new(Kind::Custom(22), content)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

fn signed_at(content: &str, created_at: u64) -> Event {
    EventBuilder::new(Kind::Custom(22), content)
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}
