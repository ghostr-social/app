//! SDK relay removal behind the role book's external-operation seam.

use crate::relay::registration::{RelayRegistration, SdkRelayRegistration};
use nostr_sdk::Client;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub(crate) type RelayRemovalFuture<'a> =
    Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

pub(crate) trait RelayRemoval: Send + Sync {
    fn remove<'a>(&'a self, url: &'a str) -> RelayRemovalFuture<'a>;
}

pub(crate) struct RelayRoleIo {
    pub(super) client: Arc<Client>,
    pub(super) removal: Arc<dyn RelayRemoval>,
    pub(super) registration: Arc<dyn RelayRegistration>,
}

impl RelayRoleIo {
    pub(crate) fn new(client: Arc<Client>, removal: Arc<dyn RelayRemoval>) -> Self {
        let registration = Arc::new(SdkRelayRegistration::new(client.clone()));
        Self {
            client,
            removal,
            registration,
        }
    }

    pub(crate) fn sdk(client: Arc<Client>) -> Self {
        let removal = Arc::new(SdkRelayRemoval {
            client: client.clone(),
        });
        Self::new(client, removal)
    }

    #[cfg(test)]
    pub(crate) fn with_registration(
        client: Arc<Client>,
        registration: Arc<dyn RelayRegistration>,
    ) -> Self {
        let removal = Arc::new(SdkRelayRemoval {
            client: client.clone(),
        });
        Self {
            client,
            removal,
            registration,
        }
    }
}

struct SdkRelayRemoval {
    client: Arc<Client>,
}

impl RelayRemoval for SdkRelayRemoval {
    fn remove<'a>(&'a self, url: &'a str) -> RelayRemovalFuture<'a> {
        Box::pin(async move {
            self.client.remove_relay(url).await?;
            Ok(())
        })
    }
}
