//! Owned Nostr SDK relay registration with responsive reconnects.

use nostr_sdk::{Client, RelayOptions, RelayServiceFlags};
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub(crate) const RELAY_RETRY_INTERVAL: Duration = Duration::from_secs(4);

#[derive(Clone, Copy)]
pub(crate) struct RelayRegistrationPolicy {
    pub(crate) flags: RelayServiceFlags,
    pub(crate) retry_interval: Duration,
    pub(crate) eager_connect: bool,
}

impl RelayRegistrationPolicy {
    pub(crate) fn eager(flags: RelayServiceFlags) -> Self {
        Self {
            flags,
            retry_interval: RELAY_RETRY_INTERVAL,
            eager_connect: true,
        }
    }
}

pub(crate) type RelayRegistrationFuture<'a> =
    Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

pub(crate) trait RelayRegistration: Send + Sync {
    fn register<'a>(
        &'a self,
        url: &'a str,
        policy: RelayRegistrationPolicy,
    ) -> RelayRegistrationFuture<'a>;

    fn forget(&self, url: &str);
}

pub(crate) struct SdkRelayRegistration {
    client: Arc<Client>,
    owned: Mutex<HashSet<String>>,
}

impl SdkRelayRegistration {
    pub(crate) fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            owned: Mutex::new(HashSet::new()),
        }
    }

    async fn own(&self, url: &str, policy: RelayRegistrationPolicy) -> anyhow::Result<()> {
        let known = self.owned.lock().expect("owned relays").contains(url);
        if self.client.relay(url).await.is_ok() && !known {
            self.client.force_remove_relay(url).await?;
        }
        if self.client.relay(url).await.is_err() {
            self.add(url, policy).await?;
        }
        self.owned
            .lock()
            .expect("owned relays")
            .insert(url.to_owned());
        Ok(())
    }

    async fn add(&self, url: &str, policy: RelayRegistrationPolicy) -> anyhow::Result<()> {
        let options = RelayOptions::new()
            .flags(policy.flags)
            .retry_interval(policy.retry_interval);
        if !self.client.pool().add_relay(url, options).await? {
            anyhow::bail!("relay registration raced for {url}");
        }
        Ok(())
    }
}

impl RelayRegistration for SdkRelayRegistration {
    fn register<'a>(
        &'a self,
        url: &'a str,
        policy: RelayRegistrationPolicy,
    ) -> RelayRegistrationFuture<'a> {
        Box::pin(async move {
            self.own(url, policy).await?;
            self.client.relay(url).await?.flags().add(policy.flags);
            if policy.eager_connect {
                self.client.connect_relay(url).await?;
            }
            Ok(())
        })
    }

    fn forget(&self, url: &str) {
        self.owned.lock().expect("owned relays").remove(url);
    }
}
