//! Atomic manager-owned reset used by the debug dashboard.

use crate::video::delivery_manager::DeliveryWorker;

impl DeliveryWorker {
    pub(crate) async fn clear(&mut self) -> anyhow::Result<()> {
        self.commands.discard_pending();
        self.downloads.clear();
        self.queue.clear();
        self.probes.clear();
        self.retry.clear();
        self.pending_demand = None;
        self.state.clear();
        self.cache.replace(Vec::new());
        self.ctx.store.clear().await
    }
}
