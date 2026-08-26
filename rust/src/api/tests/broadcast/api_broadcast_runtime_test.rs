//! Accepted API broadcasts cross relay IO and enter the local event pool.

use crate::api::runtime::discovery::{DiscoveryBoot, DiscoveryRuntime};
use ghostr_discovery::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use ghostr_discovery::test_support::TestRelayIo;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::DataUsageLevel;
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
    let (_demand_sender, demand) = watch::channel(DiscoveryDemand::Hold);
    let mut runtime = DiscoveryRuntime::start(DiscoveryBoot {
        client: std::sync::Arc::clone(&client),
        demand,
        bootstrap: vec![RELAY.to_owned()],
        search_relays: Vec::new(),
        candidates: None,
    })
    .await;
    let io = Arc::new(TestRelayIo::blocked());
    runtime.relay_pool = Arc::new(RelayPoolOwner::with_io(
        client,
        RelayPoolConfiguration {
            read_relays: vec![RELAY.to_owned()],
            search_relays: Vec::new(),
        },
        Arc::<TestRelayIo>::clone(&io),
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
    let cached = runtime
        .executor
        .cache()
        .stored_for(session, &Filter::new())
        .await
        .expect("current session");
    assert!(cached.iter().any(|known| known.id == event.id));
}
