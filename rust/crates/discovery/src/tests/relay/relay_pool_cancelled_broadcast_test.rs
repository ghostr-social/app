use crate::relay::pool::{RelayBroadcastRequest, RelayPoolConfiguration, RelayPoolOwner};
use crate::session_generation::SessionGeneration;
use crate::test_support::TestRelayIo;
use core::time::Duration;
use nostr_sdk::{Client, EventBuilder, Keys, Kind};
use std::sync::Arc;

const TRANSIENT_RELAY: &str = "wss://cancelled-broadcast.example";

#[tokio::test]
async fn cancelling_a_broadcast_releases_its_transient_relay_role() {
    let keys = Keys::generate();
    let client = Arc::new(Client::default());
    let io = Arc::new(TestRelayIo::blocked());
    let owner = Arc::new(RelayPoolOwner::with_io(
        std::sync::Arc::clone(&client),
        RelayPoolConfiguration::default(),
        std::sync::Arc::<TestRelayIo>::clone(&io),
    ));
    let mut reset = owner.begin_reset().await;
    reset
        .reset_session(SessionGeneration::initial(), Some(keys.public_key()))
        .await;
    drop(reset);
    let task_owner = std::sync::Arc::clone(&owner);
    let task = tokio::spawn(async move {
        task_owner
            .broadcast(RelayBroadcastRequest {
                session: SessionGeneration::initial(),
                relays: vec![TRANSIENT_RELAY.to_owned()],
                event: EventBuilder::new(Kind::TextNote, "hello")
                    .sign_with_keys(&keys)
                    .expect("event"),
            })
            .await
    });
    io.send_started.notified().await;

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
