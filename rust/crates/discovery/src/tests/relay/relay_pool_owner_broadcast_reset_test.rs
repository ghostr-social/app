//! Reset waits for an old send, then rejects that author in the new session.

use crate::relay::pool::{RelayBroadcastRequest, RelayPoolConfiguration, RelayPoolOwner};
use crate::session_generation::SessionGeneration;
use crate::test_support::TestRelayIo;
use nostr_sdk::{Client, EventBuilder, Keys, Kind};
use std::sync::Arc;
use tokio::sync::oneshot;

const RELAY: &str = "wss://write.example";

#[tokio::test]
async fn captured_old_signer_cannot_cross_completed_reset() {
    let old = Keys::generate();
    let fresh = Keys::generate();
    let client = Arc::new(Client::default());
    let io = Arc::new(TestRelayIo::blocked());
    let owner = Arc::new(RelayPoolOwner::with_io(
        client,
        RelayPoolConfiguration::default(),
        std::sync::Arc::<TestRelayIo>::clone(&io),
    ));
    set_account(&owner, SessionGeneration::initial(), &old).await;
    let event = EventBuilder::new(Kind::TextNote, "old")
        .sign_with_keys(&old)
        .expect("event");
    let send_owner = std::sync::Arc::clone(&owner);
    let old_send = tokio::spawn({
        let event = event.clone();
        async move {
            send_owner
                .broadcast(request(event, SessionGeneration::initial()))
                .await
        }
    });
    io.send_started.notified().await;

    let reset_owner = std::sync::Arc::clone(&owner);
    let (finished, mut reset_done) = oneshot::channel();
    tokio::spawn(async move {
        set_account(&reset_owner, SessionGeneration::initial().next(), &fresh).await;
        let _ = finished.send(());
    });
    tokio::task::yield_now().await;
    assert!(reset_done.try_recv().is_err());
    io.release_send();
    assert!(old_send.await.expect("send task").is_err());
    reset_done.await.expect("reset completes after send");

    let stale = owner
        .broadcast(request(event, SessionGeneration::initial().next()))
        .await;
    assert!(stale.is_err());
    assert_eq!(io.send_count(), 1);
}

async fn set_account(owner: &RelayPoolOwner, session: SessionGeneration, keys: &Keys) {
    let mut transition = owner.begin_reset().await;
    transition
        .reset_session(session, Some(keys.public_key()))
        .await;
}

fn request(event: nostr_sdk::Event, session: SessionGeneration) -> RelayBroadcastRequest {
    RelayBroadcastRequest {
        session,
        relays: vec![RELAY.to_owned()],
        event,
    }
}
