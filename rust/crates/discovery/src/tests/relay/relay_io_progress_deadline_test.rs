//! Progress backpressure cannot outlive a relay query deadline.

use crate::relay::health::RelayHealth;
use crate::relay::io::{RelayIo, RelayReadIo, SdkRelayIo};
use crate::tests::relay_io_relay_fixture::relay_serving;
use nostr_sdk::{Client, EventBuilder, Filter, Keys, Kind};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test]
async fn full_progress_channel_does_not_hold_the_relay_read() {
    let event = EventBuilder::new(Kind::TextNote, "event")
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let relay = relay_serving(event.clone()).await;
    let client = Arc::new(Client::default());
    client.add_relay(&relay).await.expect("mock relay");
    client.connect_relay(&relay).await.expect("connect relay");
    timeout(Duration::from_secs(1), async {
        while !client.relay(&relay).await.expect("relay").is_connected() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("relay connects");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));
    let health = Arc::new(RelayHealth::new());
    let relay_candidates = vec![relay.clone()];
    let (progress, _updates) = mpsc::channel(1);
    progress.send(event.clone()).await.expect("fill progress");

    let read = io.read(RelayReadIo {
        relays: vec![relay],
        filter: Filter::new(),
        timeout: Duration::from_millis(20),
        progress: Some(progress),
        admissions: Some(health.batch(&relay_candidates)),
    });
    let result = timeout(Duration::from_secs(2), read)
        .await
        .expect("query deadline must cover progress delivery")
        .expect("safe event remains usable");

    assert_eq!(result.events, vec![event]);
    assert!(!result.complete, "an incomplete page must remain retryable");
    assert_eq!(health.admit(&relay_candidates).len(), 1);
}
