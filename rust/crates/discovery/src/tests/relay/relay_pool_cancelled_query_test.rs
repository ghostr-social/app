use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::test_support::{read_request, TestRelayIo};
use core::time::Duration;
use nostr_sdk::Client;
use std::sync::Arc;

const TRANSIENT_RELAY: &str = "wss://cancelled-query.example";

#[tokio::test]
async fn cancelling_a_read_releases_its_transient_relay_role() {
    let client = Arc::new(Client::default());
    let io = Arc::new(TestRelayIo::blocked());
    let owner = Arc::new(RelayPoolOwner::with_io(
        std::sync::Arc::clone(&client),
        RelayPoolConfiguration::default(),
        std::sync::Arc::<TestRelayIo>::clone(&io),
    ));
    let task_owner = std::sync::Arc::clone(&owner);
    let task = tokio::spawn(async move { task_owner.read(read_request(TRANSIENT_RELAY)).await });
    io.query_started.notified().await;

    task.abort();
    let _ = task.await;

    tokio::time::timeout(Duration::from_secs(1), async {
        while client.relay(TRANSIENT_RELAY).await.is_ok() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("transient relay released after cancellation");
}
