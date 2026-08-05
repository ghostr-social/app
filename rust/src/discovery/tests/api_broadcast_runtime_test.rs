//! Accepted API broadcasts cross relay IO and enter the local event pool.

use super::relay_pool_owner_support::TestRelayIo;
use crate::api::feed_runtime::{DiscoveryBoot, DiscoveryRuntime};
use crate::discovery::relay_pool_owner::{RelayPoolConfiguration, RelayPoolOwner};
use crate::engine::inventory_controller::Mode;
use crate::engine::DataUsageLevel;
use nostr_sdk::{Client, EventBuilder, Filter, Keys, Kind};
use std::sync::Arc;
use tokio::sync::watch;

const RELAY: &str = "wss://write.example";

#[tokio::test]
async fn accepted_broadcast_is_sent_and_immediately_queryable_locally() {
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::TextNote, "accepted")
        .sign_with_keys(&keys)
        .expect("signed event");
    let client = Arc::new(Client::default());
    let (_modes, mode_updates) = watch::channel(Mode::Comfort);
    let mut runtime = DiscoveryRuntime::start(DiscoveryBoot {
        client: client.clone(),
        modes: mode_updates,
        bootstrap: vec![RELAY.to_owned()],
        search_relays: Vec::new(),
    })
    .await;
    let io = Arc::new(TestRelayIo::blocked());
    runtime.relay_pool = Arc::new(RelayPoolOwner::with_io(
        client,
        RelayPoolConfiguration {
            read_relays: vec![RELAY.to_owned()],
            search_relays: Vec::new(),
        },
        io.clone(),
    ));
    runtime.reset_session(Some(keys.public_key())).await;
    let session = runtime.session_generation();
    io.release_send();

    runtime
        .broadcast(session, event.clone(), DataUsageLevel::Balanced)
        .await
        .expect("accepted broadcast");
    runtime.remember_accepted(session, &event).await;

    assert_eq!(io.send_count(), 1);
    let cached = runtime.executor.cache().stored(&Filter::new()).await;
    assert!(cached.iter().any(|known| known.id == event.id));
}
