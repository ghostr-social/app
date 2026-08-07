//! Exclusive live-config and account-session transitions for relay ownership.

use crate::relay_pool_owner::{locked, RelayPoolOwner};
use crate::relay_pool_roles::RelayPoolConfiguration;
use crate::session_generation::SessionGeneration;
use nostr_sdk::PublicKey;
use tokio::sync::{OwnedMutexGuard, OwnedRwLockWriteGuard};

pub struct RelayPoolTransition {
    owner: RelayPoolOwner,
    _serial: OwnedMutexGuard<()>,
    _barrier: OwnedRwLockWriteGuard<()>,
    reset: bool,
}

struct ResetIntent {
    owner: RelayPoolOwner,
    active: bool,
}

impl RelayPoolOwner {
    pub async fn begin_configuration(&self) -> RelayPoolTransition {
        let serial = self.transition_serial.clone().lock_owned().await;
        let barrier = self.barrier.clone().write_owned().await;
        RelayPoolTransition {
            owner: self.clone(),
            _serial: serial,
            _barrier: barrier,
            reset: false,
        }
    }

    pub async fn begin_reset(&self) -> RelayPoolTransition {
        let intent = ResetIntent::new(self.clone());
        let serial = self.transition_serial.clone().lock_owned().await;
        let barrier = self.barrier.clone().write_owned().await;
        intent.complete(serial, barrier)
    }
}

impl RelayPoolTransition {
    pub async fn replace_configuration(&mut self, configuration: RelayPoolConfiguration) {
        debug_assert!(!self.reset);
        self.owner.roles.replace_configuration(configuration).await;
    }

    pub async fn reset_session(
        &mut self,
        session: SessionGeneration,
        expected_account: Option<PublicKey>,
    ) {
        debug_assert!(self.reset);
        self.owner.roles.reset_session().await;
        let mut lifecycle = locked(&self.owner.lifecycle);
        lifecycle.session = session;
        lifecycle.expected_account = expected_account;
    }
}

impl Drop for RelayPoolTransition {
    fn drop(&mut self) {
        if self.reset {
            let mut lifecycle = locked(&self.owner.lifecycle);
            lifecycle.pending_resets = lifecycle.pending_resets.saturating_sub(1);
        }
    }
}

impl ResetIntent {
    fn new(owner: RelayPoolOwner) -> Self {
        let epoch = {
            let mut lifecycle = locked(&owner.lifecycle);
            lifecycle.pending_resets += 1;
            lifecycle.epoch = lifecycle.epoch.wrapping_add(1);
            lifecycle.epoch
        };
        owner.cancellations.send_replace(epoch);
        Self {
            owner,
            active: true,
        }
    }

    fn complete(
        mut self,
        serial: OwnedMutexGuard<()>,
        barrier: OwnedRwLockWriteGuard<()>,
    ) -> RelayPoolTransition {
        self.active = false;
        RelayPoolTransition {
            owner: self.owner.clone(),
            _serial: serial,
            _barrier: barrier,
            reset: true,
        }
    }
}

impl Drop for ResetIntent {
    fn drop(&mut self) {
        if self.active {
            let mut lifecycle = locked(&self.owner.lifecycle);
            lifecycle.pending_resets = lifecycle.pending_resets.saturating_sub(1);
        }
    }
}
