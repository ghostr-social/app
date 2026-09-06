use super::{MediaRequestGate, RequestLease};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;

impl RequestLease {
    pub(super) fn new(
        gate: MediaRequestGate,
        authority: RequestAuthority,
        priority: PreemptionAuthority,
    ) -> Self {
        Self {
            gate,
            authority,
            priority,
            armed: true,
            body: None,
        }
    }
}

impl Drop for RequestLease {
    fn drop(&mut self) {
        if self.defer_retirement() {
            return;
        }
        if self.armed {
            self.gate.release(&self.authority, self.priority);
        }
    }
}
