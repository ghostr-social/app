//! Real discovery runtime with no external relays for API-side tests.

use crate::api::feed_runtime::{DiscoveryBoot, DiscoveryRuntime};
use crate::engine::inventory_controller::Mode;
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::watch;

pub(crate) async fn runtime() -> DiscoveryRuntime {
    let (_modes, mode_updates) = watch::channel(Mode::Comfort);
    DiscoveryRuntime::start(DiscoveryBoot {
        client: Arc::new(Client::default()),
        modes: mode_updates,
        bootstrap: Vec::new(),
        search_relays: Vec::new(),
        candidates: None,
    })
    .await
}
