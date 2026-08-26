use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner, RelayReadRequest};
use crate::test_support::read_request;
use crate::tests::relay_io_delayed_fixture::{delayed_relay, incomplete_relay};
use core::time::Duration;
use nostr_sdk::{Client, EventBuilder, Keys, Kind};
use std::sync::Arc;

#[tokio::test]
async fn owner_sdk_retry_stays_incomplete_until_every_requested_relay_recovers() {
    let event = EventBuilder::new(Kind::Custom(22), "partial")
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let healthy = delayed_relay(Some(event.clone()), Duration::ZERO).await;
    let failing = incomplete_relay(event.clone()).await;
    let owner = RelayPoolOwner::new(
        Arc::new(Client::default()),
        RelayPoolConfiguration::default(),
    );

    let first = owner
        .read(request(&healthy, &failing))
        .await
        .expect("partial union");
    let retry = owner
        .read(request(&healthy, &failing))
        .await
        .expect("healthy admitted set");

    assert_eq!(first.events, vec![event.clone()]);
    assert!(!first.complete);
    assert_eq!(retry.events, vec![event]);
    assert!(!retry.complete);
}

fn request(healthy: &str, failing: &str) -> RelayReadRequest {
    let mut request = read_request(healthy);
    request.relays = Some(vec![healthy.to_owned(), failing.to_owned()]);
    request.query.timeout = Duration::from_millis(50);
    request
}
