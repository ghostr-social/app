//! Live Nostr relay replacement for the installed discovery runtime.

use crate::api::runtime::discovery::DiscoveryRuntime;
use crate::discovery::relay::pool::{RelayPoolConfiguration, RelayPoolOwner, RelayPoolTransition};
use nostr_sdk::Client;
use std::sync::Arc;

pub(crate) async fn initialize_relay_pool(
    client: Arc<Client>,
    read_relays: Vec<String>,
    search_relays: Vec<String>,
) -> Arc<RelayPoolOwner> {
    let configuration = configuration(read_relays, search_relays);
    let owner = Arc::new(RelayPoolOwner::new(client, configuration.clone()));
    let mut transition = owner.begin_configuration().await;
    transition.replace_configuration(configuration).await;
    drop(transition);
    owner
}

pub(crate) async fn replace_relays(
    runtime: &DiscoveryRuntime,
    transition: &mut RelayPoolTransition,
    read_relays: Vec<String>,
    search_relays: Vec<String>,
) {
    runtime
        .outbox
        .write()
        .await
        .replace_bootstrap(read_relays.clone());
    runtime.executor.set_search_relays(search_relays.clone());
    transition
        .replace_configuration(configuration(read_relays, search_relays))
        .await;
}

fn configuration(read_relays: Vec<String>, search_relays: Vec<String>) -> RelayPoolConfiguration {
    RelayPoolConfiguration {
        read_relays,
        search_relays,
    }
}
