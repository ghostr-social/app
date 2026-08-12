//! The runtime exposes the account-session generation owned by feed state.

use crate::api::runtime::discovery::{DiscoveryBoot, DiscoveryRuntime};
use crate::discovery::session_generation::SessionGeneration;
use crate::engine::adaptive::DiscoveryDemand;
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::watch;

#[tokio::test]
async fn reset_advances_the_runtime_session_generation() {
    let (_demand_sender, demand) = watch::channel(DiscoveryDemand::Hold);
    let runtime = DiscoveryRuntime::start(DiscoveryBoot {
        client: Arc::new(Client::default()),
        demand,
        bootstrap: Vec::new(),
        search_relays: Vec::new(),
        candidates: None,
    })
    .await;

    assert_eq!(runtime.session_generation(), SessionGeneration::initial());
    runtime.reset_session(None).await;
    assert_eq!(
        runtime.session_generation(),
        SessionGeneration::initial().next()
    );
}
