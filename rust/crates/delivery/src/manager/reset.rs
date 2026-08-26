//! Atomic manager-owned reset used by the debug dashboard.

use crate::manager::DeliveryWorker;

impl DeliveryWorker {
    pub(super) async fn clear(&mut self) -> anyhow::Result<()> {
        self.commands.discard_pending();
        self.cancel_all_transforms();
        self.downloads.clear();
        self.queue.clear();
        self.additional_request_slot_demand = None;
        self.probes.clear();
        self.retry.clear();
        self.cooldown_timers.clear();
        self.network_refill_timer.clear();
        self.pressure.clear();
        self.hedge_tail_timers.clear();
        self.demand_leases.clear();
        self.segmented.clear();
        self.timelines.clear();
        self.independent_objects.clear();
        self.whole_body_limits.clear();
        self.state.clear();
        self.focus_lease.pin(self.ctx.store.as_ref(), None);
        self.cache.replace(Vec::new());
        self.ctx.store.clear().await
    }
}
