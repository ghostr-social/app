//! A stable configuration route held through its relay network operations.

use crate::plan_executor::PlanFailure;
use crate::relay_io::{RelayBroadcastIo, RelayReadIo};
use crate::relay_pool_owner::{
    session_failure, RelayBroadcastRequest, RelayPoolOwner, RelayReadRequest,
};
use crate::relay_pool_roles::RelayRole;
use std::sync::Arc;
use tokio::sync::{watch, OwnedRwLockReadGuard};

const NO_RELAYS_MESSAGE: &str = "no Nostr relays are configured";

pub struct RelayPoolRoute {
    owner: RelayPoolOwner,
    session: crate::session_generation::SessionGeneration,
    epoch: u64,
    cancellation: watch::Receiver<u64>,
    _barrier: OwnedRwLockReadGuard<()>,
}

impl RelayPoolOwner {
    pub async fn begin_route(
        &self,
        session: crate::session_generation::SessionGeneration,
    ) -> Result<Arc<RelayPoolRoute>, PlanFailure> {
        let cancellation = self.cancellations.subscribe();
        let barrier = self.barrier.clone().read_owned().await;
        let epoch = *cancellation.borrow();
        self.ensure_session(session, epoch)?;
        Ok(Arc::new(RelayPoolRoute {
            owner: self.clone(),
            session,
            epoch,
            cancellation,
            _barrier: barrier,
        }))
    }
}

impl RelayPoolRoute {
    pub(crate) async fn read(
        &self,
        request: RelayReadRequest,
    ) -> Result<Vec<nostr_sdk::Event>, PlanFailure> {
        self.ensure_request(request.session)?;
        let relays = self.read_targets(request.relays.clone()).await;
        if relays.is_empty() {
            return Err(PlanFailure::new(NO_RELAYS_MESSAGE));
        }
        let leased = self.owner.roles.acquire(&relays, RelayRole::Read).await;
        let result = self.read_io(leased.clone(), request).await;
        self.owner.roles.release(&leased, RelayRole::Read).await;
        self.ensure_current()?;
        result
    }

    pub async fn broadcast(&self, request: RelayBroadcastRequest) -> anyhow::Result<()> {
        self.ensure_request(request.session)
            .map_err(|failure| anyhow::anyhow!(failure.message))?;
        self.owner.ensure_author(&request.event.pubkey)?;
        anyhow::ensure!(!request.relays.is_empty(), NO_RELAYS_MESSAGE);
        let leased = self
            .owner
            .roles
            .acquire(&request.relays, RelayRole::Write)
            .await;
        let result = self
            .owner
            .io
            .broadcast(RelayBroadcastIo {
                relays: leased.clone(),
                event: request.event,
            })
            .await;
        self.owner.roles.release(&leased, RelayRole::Write).await;
        self.ensure_current()
            .map_err(|failure| anyhow::anyhow!(failure.message))?;
        result
    }

    pub(crate) fn ensure_current(&self) -> Result<(), PlanFailure> {
        self.owner.ensure_session(self.session, self.epoch)
    }

    async fn read_targets(&self, relays: Option<Vec<String>>) -> Vec<String> {
        match relays {
            Some(relays) => relays,
            None => self.owner.roles.fallback_read_relays().await,
        }
    }

    async fn read_io(
        &self,
        relays: Vec<String>,
        request: RelayReadRequest,
    ) -> Result<Vec<nostr_sdk::Event>, PlanFailure> {
        let mut cancellation = self.cancellation.clone();
        let io = RelayReadIo {
            relays,
            filter: request.query.filter,
            timeout: request.query.timeout,
            progress: request.progress,
        };
        tokio::select! {
            biased;
            _ = cancellation.changed() => Err(session_failure()),
            result = self.owner.io.read(io) => {
                result.map_err(|error| PlanFailure::new(error.to_string()))
            },
        }
    }

    fn ensure_request(
        &self,
        session: crate::session_generation::SessionGeneration,
    ) -> Result<(), PlanFailure> {
        if session != self.session {
            return Err(session_failure());
        }
        self.ensure_current()
    }
}
