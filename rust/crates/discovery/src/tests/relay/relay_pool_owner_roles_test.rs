//! Concurrent explicit read/write leases clean only their own relay role.

use crate::relay::pool::{RelayBroadcastRequest, RelayPoolConfiguration, RelayPoolOwner};
use crate::relay::removal::RelayRoleIo;
use crate::relay::roles::{RelayPoolRoles, RelayRole};
use crate::session_generation::SessionGeneration;
use crate::test_support::{read_request, TestRelayIo};
use nostr_sdk::{Client, EventBuilder, Keys, Kind};
use std::sync::Arc;

const DYNAMIC: &str = "wss://old-account.example";

#[tokio::test]
async fn shared_dynamic_relay_leaves_after_both_operations_finish() {
    let keys = Keys::generate();
    let client = Arc::new(Client::default());
    let io = Arc::new(TestRelayIo::blocked());
    let owner = Arc::new(RelayPoolOwner::with_io(
        std::sync::Arc::clone(&client),
        RelayPoolConfiguration::default(),
        std::sync::Arc::<TestRelayIo>::clone(&io),
    ));
    let mut transition = owner.begin_reset().await;
    transition
        .reset_session(SessionGeneration::initial(), Some(keys.public_key()))
        .await;
    drop(transition);

    let read_owner = std::sync::Arc::clone(&owner);
    let read = tokio::spawn(async move { read_owner.read(read_request(DYNAMIC)).await });
    io.query_started.notified().await;
    let send_owner = std::sync::Arc::clone(&owner);
    let send = tokio::spawn(async move {
        send_owner
            .broadcast(RelayBroadcastRequest {
                session: SessionGeneration::initial(),
                relays: vec![DYNAMIC.to_owned()],
                event: EventBuilder::new(Kind::TextNote, "hello")
                    .sign_with_keys(&keys)
                    .expect("event"),
            })
            .await
    });
    io.send_started.notified().await;

    let flags = client.relay(DYNAMIC).await.expect("shared relay");
    assert!(flags.flags().has_read() && flags.flags().has_write());
    io.release_query();
    assert!(read.await.expect("read task").is_ok());
    let flags = client.relay(DYNAMIC).await.expect("write lease remains");
    assert!(!flags.flags().has_read() && flags.flags().has_write());
    io.release_send();
    assert!(send.await.expect("send task").is_ok());
    assert!(client.relay(DYNAMIC).await.is_err());
}

#[tokio::test]
async fn invalid_and_unowned_write_roles_leave_the_pool_clean() {
    let client = Arc::new(Client::default());
    let roles = RelayPoolRoles::new(
        RelayRoleIo::sdk(std::sync::Arc::clone(&client)),
        RelayPoolConfiguration::default(),
    );

    roles
        .release(&["wss://never-owned.example".to_owned()], RelayRole::Write)
        .await;
    roles
        .acquire(&["not a relay".to_owned()], RelayRole::Write)
        .await;

    assert!(client.relays().await.is_empty());
}
