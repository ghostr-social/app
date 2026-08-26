use super::DeliveryState;
use crate::delivery_events::DeliveryNetworkStatus;
use ghostr_engine::origin_model::NetworkClass;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NetworkStatusUpdate {
    Stale,
    Refreshed,
    ClassChanged,
}

impl DeliveryState {
    pub(crate) const fn network_status(&self) -> DeliveryNetworkStatus {
        self.network_status
    }

    pub(crate) const fn network_profile_generation(&self) -> u64 {
        self.network_profile_generation
    }

    pub(crate) fn apply_network_status(
        &mut self,
        status: DeliveryNetworkStatus,
    ) -> NetworkStatusUpdate {
        if !status.is_fresher_than(self.network_status) {
            return NetworkStatusUpdate::Stale;
        }
        let class_changed = status.network_class() != self.network_status.network_class();
        self.network_status = status;
        if class_changed {
            NetworkStatusUpdate::ClassChanged
        } else {
            NetworkStatusUpdate::Refreshed
        }
    }

    pub(crate) fn network_class(&self) -> NetworkClass {
        self.network_status.network_class()
    }

    pub(crate) fn apply_network_profile_generation(&mut self, generation: u64) -> bool {
        if generation <= self.network_profile_generation {
            return false;
        }
        self.network_profile_generation = generation;
        true
    }
}
