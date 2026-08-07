//! A transient SDK removal failure leaves role cleanup retryable.

use crate::relay_pool_roles::{RelayPoolConfiguration, RelayPoolRoles, RelayRole};
use crate::relay_removal::{RelayRemoval, RelayRemovalFuture, RelayRoleIo};
use nostr_sdk::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const RELAY: &str = "wss://retry-removal.example";

struct FailOnceRemoval {
    client: Arc<Client>,
    failed: AtomicBool,
}

impl RelayRemoval for FailOnceRemoval {
    fn remove<'a>(&'a self, url: &'a str) -> RelayRemovalFuture<'a> {
        Box::pin(async move {
            if !self.failed.swap(true, Ordering::SeqCst) {
                anyhow::bail!("fixture removal failure");
            }
            self.client.remove_relay(url).await?;
            Ok(())
        })
    }
}

#[tokio::test]
async fn failed_removal_is_retried_after_the_next_lease() {
    let client = Arc::new(Client::default());
    let removal = Arc::new(FailOnceRemoval {
        client: client.clone(),
        failed: AtomicBool::new(false),
    });
    let io = RelayRoleIo::new(client.clone(), removal);
    let roles = RelayPoolRoles::new(io, RelayPoolConfiguration::default());
    let relays = [RELAY.to_owned()];

    roles.acquire(&relays, RelayRole::Read).await;
    roles.release(&relays, RelayRole::Read).await;
    assert!(client.relay(RELAY).await.is_ok());

    roles.acquire(&relays, RelayRole::Read).await;
    roles.release(&relays, RelayRole::Read).await;
    assert!(client.relay(RELAY).await.is_err());
}
