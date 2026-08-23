//! A single relay receives the same deterministic union ordering as fan-out reads.

use crate::relay::io::{RelayIo, RelayReadIo, SdkRelayIo};
use crate::tests::relay_io_relay_fixture::relay_serving_events;
use nostr_sdk::{Client, Event, EventBuilder, Filter, Keys, Kind, Timestamp};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn single_relay_frames_are_returned_newest_first() {
    let older = signed_at("older", 10);
    let newer = signed_at("newer", 20);
    let relay = relay_serving_events(vec![older.clone(), newer.clone()]).await;
    let client = Arc::new(Client::default());
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
        .expect("single relay result");

    assert_eq!(result.events, vec![newer, older]);
    assert!(result.complete);
}

fn signed_at(content: &str, created_at: u64) -> Event {
    EventBuilder::new(Kind::Custom(22), content)
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}
