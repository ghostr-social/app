use super::PartialRangeStore;
use anyhow::{ensure, Result};
use ghostr_engine::representation::TransferIdentity;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod accounting;
mod extension;
pub use extension::ActionReservationExtension;

#[derive(Clone, Debug)]
pub struct StoreAction {
    id: u64,
    key: String,
    identity: TransferIdentity,
    active: Arc<AtomicBool>,
    events: super::capacity::CapacityEvents,
}

pub(super) struct ActionReservation {
    key: String,
    identity: TransferIdentity,
    remaining: u64,
    active: Arc<AtomicBool>,
}

impl StoreAction {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn identity(&self) -> &TransferIdentity {
        &self.identity
    }

    pub fn revoke(&self) {
        if self.active.swap(false, Ordering::AcqRel) {
            self.events.signal();
        }
    }

    pub fn same_authority(&self, other: &Self) -> bool {
        self.id == other.id
            && self.key == other.key
            && self.identity == other.identity
            && Arc::ptr_eq(&self.active, &other.active)
    }
}

impl PartialRangeStore {
    pub async fn reserve_action(
        &self,
        identity: &TransferIdentity,
        id: u64,
        maximum_bytes: u64,
    ) -> Result<StoreAction> {
        ensure!(maximum_bytes > 0, "action reservation must be positive");
        let key = identity.post().as_str().to_owned();
        self.retry_inactive_single_response(&key).await;
        self.retry_inactive_sparse_responses(&key).await?;
        self.retry_cleanup_debt(&key).await?;
        let _update = self.update_key(&key).await?;
        let _capacity = self.capacity_updates.lock().await;
        self.current_binding(identity).await?;
        ensure!(
            !self.action_reservations.lock().await.contains_key(&id),
            "action reservation already exists"
        );
        let mut entries = self.entries.lock().await;
        self.make_room(&mut entries, &key, maximum_bytes).await?;
        let active = Arc::new(AtomicBool::new(true));
        let reservation = ActionReservation {
            key: key.clone(),
            identity: identity.clone(),
            remaining: maximum_bytes,
            active: active.clone(),
        };
        self.action_reservations
            .lock()
            .await
            .insert(id, reservation);
        Ok(StoreAction {
            id,
            key,
            identity: identity.clone(),
            active,
            events: self.capacity.events(),
        })
    }

    pub async fn release_action(&self, action: &StoreAction) {
        action.revoke();
        let cleanup = self.abort_response_for_action(action).await;
        if cleanup.is_err() {
            self.quarantine_failed_sparse_action(action).await;
        }
        let _capacity = self.capacity_updates.lock().await;
        let mut reservations = self.action_reservations.lock().await;
        let matches = reservations
            .get(&action.id)
            .is_some_and(|reservation| same_authority(reservation, action));
        let released = matches
            .then(|| reservations.remove(&action.id))
            .flatten()
            .map_or(0, |reservation| reservation.remaining);
        drop(reservations);
        self.capacity.released_reservation(released);
        if let Err(error) = cleanup {
            log::warn!("Could not clean up video action {}: {error:#}", action.id);
        }
    }

    pub(super) async fn revoke_all_actions(&self) {
        let mut reservations = self.action_reservations.lock().await;
        let released = reservations
            .drain()
            .map(|(_, reservation)| {
                reservation.active.store(false, Ordering::Release);
                reservation.remaining
            })
            .sum();
        drop(reservations);
        self.capacity.released_reservation(released);
    }

    pub(super) async fn consume_action(&self, action: &StoreAction, bytes: u64) -> Result<u64> {
        let mut reservations = self.action_reservations.lock().await;
        let reservation = reservations
            .get_mut(&action.id)
            .filter(|reservation| same_authority(reservation, action))
            .ok_or_else(|| anyhow::anyhow!("action reservation is missing"))?;
        ensure!(
            reservation.active.load(Ordering::Acquire),
            "action reservation was revoked"
        );
        ensure!(
            reservation.remaining >= bytes,
            "action exceeded its reservation"
        );
        reservation.remaining -= bytes;
        Ok(bytes)
    }

    pub(super) async fn charge_action_write(&self, action: &StoreAction, bytes: u64) -> Result<()> {
        let mut used = self.used_bytes.lock().await;
        let mut reservations = self.action_reservations.lock().await;
        let reservation = reservations
            .get_mut(&action.id)
            .filter(|reservation| same_authority(reservation, action))
            .ok_or_else(|| anyhow::anyhow!("action reservation is missing"))?;
        ensure!(
            reservation.active.load(Ordering::Acquire),
            "action was revoked"
        );
        ensure!(
            reservation.remaining >= bytes,
            "action exceeded its reservation"
        );
        reservation.remaining -= bytes;
        *used = used.saturating_add(bytes);
        drop(reservations);
        drop(used);
        self.capacity.spent(bytes).await;
        Ok(())
    }
}

pub(super) type ActionReservations = HashMap<u64, ActionReservation>;

fn same_authority(reservation: &ActionReservation, action: &StoreAction) -> bool {
    reservation.key == action.key
        && reservation.identity == action.identity
        && Arc::ptr_eq(&reservation.active, &action.active)
}
