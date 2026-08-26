//! Reset cancellation is not trapped behind an already queued config writer.

use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::session_generation::SessionGeneration;
use crate::test_support::{read_request, TestRelayIo};
use core::time::Duration;
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::time::timeout;

const DYNAMIC: &str = "wss://old-account.example";

#[tokio::test]
async fn reset_cancels_a_query_a_configuration_writer_is_waiting_for() {
    let io = Arc::new(TestRelayIo::blocked());
    let owner = Arc::new(RelayPoolOwner::with_io(
        Arc::new(Client::default()),
        RelayPoolConfiguration::default(),
        std::sync::Arc::<TestRelayIo>::clone(&io),
    ));
    let query_owner = std::sync::Arc::clone(&owner);
    let query = tokio::spawn(async move { query_owner.read(read_request(DYNAMIC)).await });
    io.query_started.notified().await;

    let config_owner = std::sync::Arc::clone(&owner);
    let config = tokio::spawn(async move {
        let _guard = config_owner.begin_configuration().await;
    });
    wait_until_serial_is_held(&owner).await;
    let reset_owner = std::sync::Arc::clone(&owner);
    let reset = tokio::spawn(async move {
        let mut guard = reset_owner.begin_reset().await;
        guard
            .reset_session(SessionGeneration::initial().next(), None)
            .await;
    });

    timeout(Duration::from_secs(1), reset)
        .await
        .expect("reset must not deadlock")
        .expect("reset task");
    assert!(query.await.expect("query task").is_err());
    config.await.expect("configuration task");
}

async fn wait_until_serial_is_held(owner: &RelayPoolOwner) {
    timeout(Duration::from_secs(1), async {
        loop {
            match std::sync::Arc::clone(&owner.transition_serial).try_lock_owned() {
                Ok(guard) => {
                    drop(guard);
                    tokio::task::yield_now().await;
                }
                Err(_) => return,
            }
        }
    })
    .await
    .expect("configuration must acquire transition serial");
}
