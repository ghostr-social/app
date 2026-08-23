//! Exact subscriptions still enforce their requested filter and event bound.

use crate::relay::io::{RelayIo, RelayReadIo, SdkRelayIo};
use crate::tests::relay_io_relay_fixture::relay_serving_events;
use nostr_sdk::{Client, Event, EventBuilder, Filter, Keys, Kind};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn wrong_filter_events_do_not_enter_the_answer() {
    let wrong = signed(Kind::TextNote, "wrong");
    let first = signed(Kind::Custom(22), "first");
    let second = signed(Kind::Custom(22), "second");
    let relay = relay_serving_events(vec![wrong, first.clone(), second.clone()]).await;
    let client = Arc::new(Client::default());
    client.add_relay(&relay).await.expect("mock relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));

    let result = io
        .read(RelayReadIo {
            relays: vec![relay],
            filter: Filter::new()
                .kind(Kind::Custom(22))
                .search("relay-ranked query")
                .limit(2),
            timeout: Duration::from_secs(1),
            progress: None,
            admissions: None,
        })
        .await
        .expect("bounded exact query");

    assert_eq!(result.events.len(), 2);
    assert!(result.events.contains(&first));
    assert!(result.events.contains(&second));
    assert!(result.complete);
}

#[tokio::test]
async fn known_id_with_a_forged_body_does_not_enter_the_answer() {
    let valid = signed(Kind::Custom(22), "valid");
    let mut forged = valid.clone();
    forged.content = "forged".to_owned();
    let relay = relay_serving_events(vec![forged]).await;
    let client = Arc::new(Client::default());
    client
        .database()
        .save_event(&valid)
        .await
        .expect("seed known id");
    client.add_relay(&relay).await.expect("mock relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));

    let result = io
        .read(RelayReadIo {
            relays: vec![relay],
            filter: Filter::new().kind(Kind::Custom(22)),
            timeout: Duration::from_secs(1),
            progress: None,
            admissions: None,
        })
        .await
        .expect("invalid event is ignored");

    assert!(result.events.is_empty());
    assert!(result.complete);
}

fn signed(kind: Kind, content: &str) -> Event {
    EventBuilder::new(kind, content)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}
