use super::MetadataProbePool;
use ghostr_engine::representation::TransferIdentity;
use std::collections::HashSet;

impl MetadataProbePool {
    pub fn defer_to_body(&mut self, identity: &TransferIdentity) {
        self.probing.remove(identity.post());
        self.deferred.insert(identity.clone());
    }

    pub fn require_body(&mut self, identity: &TransferIdentity) {
        self.defer_to_body(identity);
        self.head_unavailable.insert(identity.clone());
    }

    pub fn body_satisfied(&mut self, identity: &TransferIdentity) {
        self.head_unavailable.remove(identity);
    }

    pub fn body_finished(&mut self, identity: &TransferIdentity) {
        if !self.head_unavailable.contains(identity) {
            self.deferred.remove(identity);
        }
    }

    pub(crate) fn reconcile_bodies(&mut self, active: &HashSet<TransferIdentity>) {
        let unavailable = &self.head_unavailable;
        self.deferred
            .retain(|identity| active.contains(identity) || unavailable.contains(identity));
        self.head_unavailable
            .retain(|identity| self.deferred.contains(identity));
    }
}
