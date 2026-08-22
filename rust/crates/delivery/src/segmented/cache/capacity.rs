use super::{objects, staged, SegmentedCache, MAX_CACHE_BYTES};

impl SegmentedCache {
    pub(crate) fn planning_available_bytes(&self) -> u64 {
        let state = self.lock();
        physical_available(staged::physical_used(&state))
            .saturating_add(objects::reclaimable_ready_bytes(&state))
            .min(MAX_CACHE_BYTES as u64)
    }

    #[cfg(test)]
    pub(crate) fn physical_available_bytes(&self) -> u64 {
        physical_available(staged::physical_used(&self.lock()))
    }
}

fn physical_available(used: u64) -> u64 {
    (MAX_CACHE_BYTES as u64).saturating_sub(used)
}
