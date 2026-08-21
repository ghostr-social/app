use super::{same_authority, StoreAction};
use crate::partial_range_store::PartialRangeStore;
use std::collections::HashSet;

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn restore_action(
        &self,
        action: &StoreAction,
        bytes: u64,
    ) {
        let mut reservations = self.action_reservations.lock().await;
        if let Some(reservation) = reservations
            .get_mut(&action.id)
            .filter(|item| same_authority(item, action))
        {
            reservation.remaining = reservation.remaining.saturating_add(bytes);
        }
    }

    pub(in crate::partial_range_store) async fn reserved_bytes(&self) -> u64 {
        self.action_reservations
            .lock()
            .await
            .values()
            .map(|reservation| reservation.remaining)
            .sum()
    }

    pub(in crate::partial_range_store) async fn reserved_keys(&self) -> HashSet<String> {
        self.action_reservations
            .lock()
            .await
            .values()
            .map(|reservation| reservation.key.clone())
            .collect()
    }
}
