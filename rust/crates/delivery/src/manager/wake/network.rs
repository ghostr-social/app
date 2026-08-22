use crate::delivery_events::DeliveryNetworkStatus;
use crate::manager::state::network::NetworkStatusUpdate;
use crate::manager::DeliveryWorker;

impl DeliveryWorker {
    pub(super) fn apply_network_status(&mut self, status: DeliveryNetworkStatus) {
        let update = self.state.apply_network_status(status);
        if update == NetworkStatusUpdate::Stale {
            return;
        }
        self.ctx.network_status.update(status);
        if update == NetworkStatusUpdate::ClassChanged {
            self.note_network_class_change();
        }
    }
}
