//! A sibling EOSE cannot terminate another in-flight relay query.

use crate::relay::io::{RelayIo as _, RelayReadIo, RelayReadResult, SdkRelayIo};
use crate::tests::relay_io_concurrent_fixture::relay_closing_empty_before_event;
use core::time::Duration;
use nostr_sdk::{Client, EventBuilder, Filter, Keys, Kind};
use std::sync::Arc;

#[tokio::test]
async fn empty_subscription_close_does_not_terminate_video_read() {
    let event = EventBuilder::new(Kind::Custom(22), "video")
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let relay = relay_closing_empty_before_event(event.clone()).await;
    let client = Arc::new(Client::default());
    client.add_relay(&relay).await.expect("mock relay");
    let io = Arc::new(SdkRelayIo::with_readiness_timeout(
        client,
        Duration::from_secs(2),
    ));

    let video = read(std::sync::Arc::clone(&io), relay.clone(), Kind::Custom(22));
    let empty = read(io, relay, Kind::TextNote);
    let (video, empty) = tokio::join!(video, empty);

    assert_eq!(video.expect("video read").events, vec![event]);
    assert!(empty.expect("empty read").events.is_empty());
}

async fn read(io: Arc<SdkRelayIo>, relay: String, kind: Kind) -> anyhow::Result<RelayReadResult> {
    io.read(RelayReadIo {
        relays: vec![relay],
        filter: Filter::new().kind(kind),
        timeout: Duration::from_secs(1),
        progress: None,
        admissions: None,
    })
    .await
}
