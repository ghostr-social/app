//! Session reset cancels old reads and leaves no dynamic relay behind.

use super::relay_pool_owner_support::{read_request, TestRelayIo};
use crate::discovery::relay_pool_owner::{RelayPoolConfiguration, RelayPoolOwner};
use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::Client;
use std::sync::Arc;

const DYNAMIC: &str = "wss://old-account.example";

#[tokio::test]
async fn reset_finishes_without_releasing_the_old_query() {
    let client = Arc::new(Client::default());
    let io = Arc::new(TestRelayIo::blocked());
    let owner = Arc::new(RelayPoolOwner::with_io(
        client.clone(),
        RelayPoolConfiguration::default(),
        io.clone(),
    ));
    let query_owner = owner.clone();
    let query = tokio::spawn(async move { query_owner.read(read_request(DYNAMIC)).await });
    io.query_started.notified().await;

    let mut transition = owner.begin_reset().await;
    transition
        .reset_session(SessionGeneration::initial().next(), None)
        .await;
    drop(transition);

    assert!(query.await.expect("query task").is_err());
    assert!(client.relay(DYNAMIC).await.is_err());
}
