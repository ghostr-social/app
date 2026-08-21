use super::{overlaps, InFlightChunks};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ActionId, ChunkId};

impl InFlightChunks {
    #[cfg(test)]
    pub fn cancel(&mut self, chunk: &ChunkId) -> bool {
        let Some(active) = self
            .transfers
            .values_mut()
            .find(|active| overlaps(&active.chunk, chunk) && !active.cancelling)
        else {
            return false;
        };
        active.cancel();
        true
    }

    pub(crate) fn cancel_action(&mut self, action: ActionId) -> bool {
        let Some(active) = self.transfers.get_mut(&action) else {
            return false;
        };
        if active.cancelling {
            return false;
        }
        active.cancel();
        true
    }

    pub(crate) fn can_cancel_action(&self, action: ActionId) -> bool {
        self.transfers
            .get(&action)
            .is_some_and(|active| !active.cancelling)
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
