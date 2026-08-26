//! A retained SDK gossip relay must not evade the owned registration policy.

use crate::relay::registration::{
    RelayRegistration as _, RelayRegistrationPolicy, SdkRelayRegistration,
};
use nostr_sdk::{Client, RelayOptions, RelayServiceFlags};
use std::sync::Arc;

const RELAY: &str = "ws://127.0.0.1:1";

#[tokio::test]
async fn a_preexisting_gossip_relay_is_replaced_by_the_owned_policy() {
    let client = Arc::new(Client::default());
    let stale_options = RelayOptions::new()
        .flags(RelayServiceFlags::PING | RelayServiceFlags::GOSSIP)
        .retry_interval(core::time::Duration::from_secs(60));
    client
        .pool()
        .add_relay(RELAY, stale_options)
        .await
        .expect("preexisting relay");
    let stale = client.relay(RELAY).await.expect("stale relay");
    let registration = SdkRelayRegistration::new(std::sync::Arc::clone(&client));
    let policy = RelayRegistrationPolicy::eager(RelayServiceFlags::PING | RelayServiceFlags::READ);

    registration
        .register(RELAY, policy)
        .await
        .expect("owned registration");
    let registered = client.relay(RELAY).await.expect("registered relay");
    let stale_was_reused = stale.flags().has_read();
    let registered_has_policy = registered.flags().has_read();
    client
        .force_remove_relay(RELAY)
        .await
        .expect("test cleanup");

    assert!(
        !stale_was_reused && registered_has_policy,
        "SDK retained the stale GOSSIP relay instead of rebuilding it with the owned options"
    );
}
