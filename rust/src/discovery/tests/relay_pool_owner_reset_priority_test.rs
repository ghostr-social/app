//! Reset cancellation is not trapped behind an already queued config writer.

use super::relay_pool_owner_support::{read_request, TestRelayIo};
use crate::discovery::relay_pool_owner::{RelayPoolConfiguration, RelayPoolOwner};
use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

const DYNAMIC: &str = "wss://old-account.example";

#[tokio::test]
async fn reset_cancels_a_query_a_configuration_writer_is_waiting_for() {
    let io = Arc::new(TestRelayIo::blocked());
    let owner = Arc::new(RelayPoolOwner::with_io(
        Arc::new(Client::default()),
        RelayPoolConfiguration::default(),
        io.clone(),
    ));
    let query_owner = owner.clone();
    let query = tokio::spawn(async move { query_owner.read(read_request(DYNAMIC)).await });
    io.query_started.notified().await;

    let config_owner = owner.clone();
    let config = tokio::spawn(async move {
        let _guard = config_owner.begin_configuration().await;
    });
    wait_until_serial_is_held(&owner).await;
    let reset_owner = owner.clone();
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
            match owner.transition_serial.clone().try_lock_owned() {
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
