use super::InFlightChunks;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::ActionId;

impl InFlightChunks {
    pub(crate) fn cancel_action(&mut self, action: ActionId) -> bool {
        let Some(active) = self.transfers.get_mut(&action) else {
            return false;
        };
        active.cancel()
    }

    pub(crate) fn can_cancel_action(&self, action: ActionId) -> bool {
        self.transfers
            .get(&action)
            .is_some_and(|active| !active.cancelling && !active.io_finished())
    }

    pub(crate) fn cancel_obsolete(&mut self, binding: &RepresentationBinding) {
        for active in self.transfers.values_mut() {
            let current = binding.transfer(active.identity.source().as_str());
            let obsolete =
                active.chunk.post == *binding.post() && current.as_ref() != Some(&active.identity);
            if obsolete {
                active.cancel();
            }
        }
    }
}

#[cfg(test)]
#[path = "cancellation_axiom_test.rs"]
pub(crate) mod axiom_test_support;
