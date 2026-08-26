use crate::relay::registration::{
    RelayRegistration as _, RelayRegistrationPolicy, SdkRelayRegistration,
};
use nostr_sdk::{Client, RelayOptions, RelayServiceFlags};
use std::sync::Arc;

const RELAY: &str = "ws://127.0.0.1:1";

#[tokio::test]
async fn a_preexisting_relay_is_replaced_with_the_owned_policy() {
    let client = Arc::new(Client::default());
    let stale_options = RelayOptions::new().flags(RelayServiceFlags::PING);
    assert!(client
        .pool()
        .add_relay(RELAY, stale_options)
        .await
        .expect("preexisting relay"));
    let stale = client.relay(RELAY).await.expect("stale relay");
    let registration = SdkRelayRegistration::new(std::sync::Arc::clone(&client));
    let policy = RelayRegistrationPolicy::eager(RelayServiceFlags::PING | RelayServiceFlags::READ);

    registration
        .register(RELAY, policy)
        .await
        .expect("owned registration");

    assert!(!stale.flags().has_read());
    assert!(client
        .relay(RELAY)
        .await
        .expect("replacement relay")
        .flags()
        .has_read());
    client.remove_relay(RELAY).await.expect("cleanup");
}
