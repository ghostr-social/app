use super::*;

use super::super::overlaps;

use ghostr_engine::ChunkId;

impl InFlightChunks {
    pub(crate) fn cancel(&mut self, chunk: &ChunkId) -> bool {
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
}
