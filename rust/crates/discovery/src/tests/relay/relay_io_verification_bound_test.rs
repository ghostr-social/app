use crate::relay::io::{RelayIo as _, RelayReadIo, SdkRelayIo};
use crate::tests::relay_io_relay_fixture::relay_serving_events;
use core::time::Duration;
use nostr_sdk::{Client, Event, EventBuilder, Filter, Keys, Kind};
use std::sync::Arc;

#[tokio::test]
async fn forged_frame_overflow_preserves_another_relays_safe_prefix() {
    let valid = signed("known");
    let mut forged = valid.clone();
    forged.sig = signed("other signature").sig;
    let hostile = relay_serving_events(vec![forged; 9]).await;
    let healthy_event = signed("healthy");
    let healthy = relay_serving_events(vec![healthy_event.clone()]).await;
    let client = Arc::new(Client::default());
    client
        .database()
        .save_event(&valid)
        .await
        .expect("seed known id");
    client.add_relay(&hostile).await.expect("hostile relay");
    client.add_relay(&healthy).await.expect("healthy relay");
    let io = SdkRelayIo::with_readiness_timeout(client, Duration::from_secs(1));

    let result = io
        .read(RelayReadIo {
            relays: vec![hostile, healthy],
            filter: Filter::new().kind(Kind::Custom(22)).limit(2),
            timeout: Duration::from_secs(1),
            progress: None,
            admissions: None,
        })
        .await
        .expect("healthy prefix remains usable");

    assert_eq!(result.events, vec![healthy_event]);
    assert!(
        !result.complete,
        "hostile verification overflow is incomplete"
    );
}

fn signed(content: &str) -> Event {
    EventBuilder::new(Kind::Custom(22), content)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}
