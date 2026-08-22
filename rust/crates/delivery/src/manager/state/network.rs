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
    pub(crate) fn apply_network_status(
        &mut self,
        status: DeliveryNetworkStatus,
    ) -> NetworkStatusUpdate {
        if !status.is_fresher_than(self.network_status) {
            return NetworkStatusUpdate::Stale;
        }
        let class_changed = status.network_class() != self.network_status.network_class();
        self.network_status = status;
        match class_changed {
            true => NetworkStatusUpdate::ClassChanged,
            false => NetworkStatusUpdate::Refreshed,
        }
    }

    pub(crate) fn network_class(&self) -> NetworkClass {
        self.network_status.network_class()
    }
}
