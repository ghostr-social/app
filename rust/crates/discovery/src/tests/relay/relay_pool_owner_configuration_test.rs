//! Live relay replacement is serialized and authoritative.

use crate::test_support::TestRelayIo;
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::oneshot;

const FIRST: &str = "wss://first.example";
const FINAL: &str = "wss://final.example";
const UNRELATED: &str = "wss://unrelated.example";

#[tokio::test]
async fn overlapping_replacements_leave_only_the_last_configuration() {
    let client = Arc::new(Client::default());
    client.add_relay(UNRELATED).await.expect("unrelated relay");
    let owner = Arc::new(RelayPoolOwner::with_io(
        client.clone(),
        RelayPoolConfiguration::default(),
        Arc::new(TestRelayIo::blocked()),
    ));
    let mut first = owner.begin_configuration().await;
    first.replace_configuration(configuration(FIRST)).await;

    let next_owner = owner.clone();
    let (finished, mut done) = oneshot::channel();
    tokio::spawn(async move {
        let mut next = next_owner.begin_configuration().await;
        next.replace_configuration(configuration(FINAL)).await;
        let _ = finished.send(());
    });
    tokio::task::yield_now().await;
    assert!(done.try_recv().is_err());
    drop(first);
    done.await.expect("second replacement");

    assert!(client.relay(FIRST).await.is_err());
    assert!(client.relay(UNRELATED).await.is_err());
    let relay = client.relay(FINAL).await.expect("final relay");
    assert!(relay.flags().has_read());
    assert!(!relay.flags().has_write());
}

fn configuration(relay: &str) -> RelayPoolConfiguration {
    RelayPoolConfiguration {
        read_relays: vec![relay.to_owned()],
        search_relays: Vec::new(),
    }
}
