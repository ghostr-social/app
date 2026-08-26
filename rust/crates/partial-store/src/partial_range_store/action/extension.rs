use super::{same_authority, StoreAction};
use crate::partial_range_store::PartialRangeStore;
use anyhow::{ensure, Context as _, Result};

#[derive(Debug)]
#[must_use = "commit or roll back every prepared action extension"]
pub struct ActionReservationExtension {
    action: StoreAction,
    previous_bytes: u64,
    extended_bytes: u64,
}

impl ActionReservationExtension {
    pub fn additional_bytes(&self) -> u64 {
        self.extended_bytes.saturating_sub(self.previous_bytes)
    }

    pub fn commit(self) {}
}

impl PartialRangeStore {
    /// # Errors
    ///
    /// Returns an error when the action is stale, missing, or cannot reserve the added capacity.
    pub async fn extend_action(
        &self,
        action: &StoreAction,
        maximum_bytes: u64,
    ) -> Result<ActionReservationExtension> {
        ensure!(action.is_active(), "action reservation was revoked");
        let _update = self.update_key(&action.key).await?;
        let _capacity = self.capacity_updates.lock().await;
        self.current_binding(action.identity()).await?;
        let previous_bytes = self.action_remaining(action).await?;
        ensure!(
            maximum_bytes >= previous_bytes,
            "action extension cannot shrink its reservation"
        );
        self.reserve_extension(action, previous_bytes, maximum_bytes)
            .await?;
        Ok(ActionReservationExtension {
            action: action.clone(),
            previous_bytes,
            extended_bytes: maximum_bytes,
        })
    }

    /// # Errors
    ///
    /// Returns an error when the reservation changed or cannot be recovered durably.
    pub async fn rollback_action(&self, extension: ActionReservationExtension) -> Result<()> {
        let _update = self.update_key(&extension.action.key).await?;
        let _capacity = self.capacity_updates.lock().await;
        let released = extension.additional_bytes();
        let mut reservations = self.action_reservations.lock().await;
        let reservation = reservations
            .get_mut(&extension.action.id)
            .filter(|item| same_authority(item, &extension.action))
            .context("action reservation is missing")?;
        ensure!(
            reservation.remaining == extension.extended_bytes,
            "action reservation changed after extension"
        );
        reservation.remaining = extension.previous_bytes;
        drop(reservations);
        self.capacity.released_reservation(released);
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error when the action is stale, missing, or cannot be resized safely.
    pub async fn resize_action(&self, action: &StoreAction, maximum_bytes: u64) -> Result<()> {
        ensure!(action.is_active(), "action reservation was revoked");
        let _update = self.update_key(&action.key).await?;
        let _capacity = self.capacity_updates.lock().await;
        let released = self.shrink_action(action, maximum_bytes).await?;
        self.capacity.released_reservation(released);
        Ok(())
    }

    async fn action_remaining(&self, action: &StoreAction) -> Result<u64> {
        self.action_reservations
            .lock()
            .await
            .get(&action.id)
            .filter(|item| same_authority(item, action))
            .map(|item| item.remaining)
            .context("action reservation is missing")
    }

    async fn reserve_extension(
        &self,
        action: &StoreAction,
        previous_bytes: u64,
        maximum_bytes: u64,
    ) -> Result<()> {
        let additional = maximum_bytes.saturating_sub(previous_bytes);
        let mut entries = self.entries.lock().await;
        self.make_room(&mut entries, &action.key, additional)
            .await?;
        drop(entries);
        let mut reservations = self.action_reservations.lock().await;
        let reservation = reservations
            .get_mut(&action.id)
            .filter(|item| same_authority(item, action))
            .context("action reservation is missing")?;
        ensure!(
            reservation.remaining == previous_bytes,
            "action reservation changed during extension"
        );
        reservation.remaining = maximum_bytes;
        Ok(())
    }

    async fn shrink_action(&self, action: &StoreAction, maximum_bytes: u64) -> Result<u64> {
        let mut reservations = self.action_reservations.lock().await;
        let reservation = reservations
            .get_mut(&action.id)
            .filter(|item| same_authority(item, action))
            .context("action reservation is missing")?;
        ensure!(
            maximum_bytes <= reservation.remaining,
            "response exceeds its launch reservation"
        );
        let released = reservation.remaining - maximum_bytes;
        reservation.remaining = maximum_bytes;
        Ok(released)
    }
}
