//! Real discovery runtime with no external relays for API-side tests.

use crate::api::runtime::discovery::{DiscoveryBoot, DiscoveryRuntime};
use crate::engine::adaptive::DiscoveryDemand;
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::watch;

pub(crate) async fn runtime() -> DiscoveryRuntime {
    let (_demand_sender, demand) = watch::channel(DiscoveryDemand::Hold);
    DiscoveryRuntime::start(DiscoveryBoot {
        client: Arc::new(Client::default()),
        demand,
        bootstrap: Vec::new(),
        search_relays: Vec::new(),
        candidates: None,
    })
    .await
}
