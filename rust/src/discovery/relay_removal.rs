//! SDK relay removal behind the role book's external-operation seam.

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
}

impl RelayRoleIo {
    pub(crate) fn new(client: Arc<Client>, removal: Arc<dyn RelayRemoval>) -> Self {
        Self { client, removal }
    }

    pub(crate) fn sdk(client: Arc<Client>) -> Self {
        let removal = Arc::new(SdkRelayRemoval {
            client: client.clone(),
        });
        Self::new(client, removal)
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
