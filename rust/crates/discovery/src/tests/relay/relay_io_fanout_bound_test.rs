use crate::relay::io::{RelayIo, RelayReadIo, SdkRelayIo};
use nostr_sdk::{Client, Filter, Kind};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn excessive_relay_fanout_is_rejected_before_queries_start() {
    let io =
        SdkRelayIo::with_readiness_timeout(Arc::new(Client::default()), Duration::from_secs(1));
    let relays = (0..33)
        .map(|index| format!("wss://relay-{index}.example"))
        .collect();

    let error = io
        .read(RelayReadIo {
            relays,
            filter: Filter::new().kind(Kind::Custom(22)),
            timeout: Duration::from_secs(1),
            progress: None,
            admissions: None,
        })
        .await
        .expect_err("fanout above the hard request bound");

    assert!(format!("{error:#}").contains("relay fanout exceeds 32"));
}
