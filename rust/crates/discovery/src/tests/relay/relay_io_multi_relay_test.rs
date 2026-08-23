//! A faster empty relay cannot exclude a slower connected target.

use crate::relay::io::{RelayIo, RelayReadIo, SdkRelayIo};
use crate::tests::relay_io_delayed_fixture::{delayed_relay, incomplete_relay};
use nostr_sdk::{Client, EventBuilder, Filter, Keys, Kind};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn slower_target_contributes_to_the_parallel_relay_union() {
    let event = EventBuilder::new(Kind::Custom(22), "video")
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let fast = delayed_relay(None, Duration::ZERO).await;
    let slow = delayed_relay(Some(event.clone()), Duration::from_millis(50)).await;
    let client = Arc::new(Client::default());
    client.add_relay(&fast).await.expect("fast relay");
    client.add_relay(&slow).await.expect("slow relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));

    let result = io
        .read(RelayReadIo {
            relays: vec![fast, slow],
            filter: Filter::new().kind(Kind::Custom(22)),
            timeout: Duration::from_secs(1),
            progress: None,
            admissions: None,
        })
        .await
        .expect("parallel relay read");

    assert_eq!(result.events, vec![event]);
    assert!(result.complete);
}

#[tokio::test]
async fn incomplete_target_returns_the_union_without_authority() {
    let event = EventBuilder::new(Kind::Custom(22), "partial")
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let complete = delayed_relay(None, Duration::ZERO).await;
    let incomplete = incomplete_relay(event.clone()).await;
    let client = Arc::new(Client::default());
    client.add_relay(&complete).await.expect("complete relay");
    client
        .add_relay(&incomplete)
        .await
        .expect("incomplete relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));

    let result = io
        .read(RelayReadIo {
            relays: vec![complete, incomplete],
            filter: Filter::new().kind(Kind::Custom(22)),
            timeout: Duration::from_millis(100),
            progress: None,
            admissions: None,
        })
        .await
        .expect("healthy relay events remain usable");

    assert_eq!(result.events, vec![event]);
    assert!(
        !result.complete,
        "a partial relay must not advance a cursor"
    );
}
