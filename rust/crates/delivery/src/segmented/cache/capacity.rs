use super::{objects, CacheState, SegmentedCache, MAX_CACHE_BYTES};
use ghostr_engine::PostId;

impl SegmentedCache {
    pub(crate) fn planning_available_bytes(&self) -> u64 {
        let state = self.lock();
        physical_available(physical_used(&state))
            .saturating_add(objects::reclaimable_ready_bytes(&state))
            .min(MAX_CACHE_BYTES as u64)
    }

    #[cfg(test)]
    pub(crate) fn physical_available_bytes(&self) -> u64 {
        physical_available(physical_used(&self.lock()))
    }

    pub(crate) fn physical_used_bytes(&self) -> u64 {
        physical_used(&self.lock())
    }

    pub(crate) const fn capacity_bytes() -> u64 {
        MAX_CACHE_BYTES as u64
    }
}

pub(super) fn fits(state: &CacheState, post: &PostId, maximum_bytes: u64) -> bool {
    let current = state.focus.get(post).map_or(0, |record| {
        record.reserved_bytes.saturating_add(record.assembly_bytes)
    });
    let used = physical_used(state).saturating_sub(current);
    maximum_bytes <= physical_available(used)
}

fn physical_used(state: &CacheState) -> u64 {
    let staged = state
        .focus
        .values()
        .flat_map(|record| &record.staged)
        .map(|object| object.len())
        .sum::<u64>();
    let reserved = state
        .focus
        .values()
        .map(|record| record.reserved_bytes)
        .sum::<u64>();
    let assembly = state
        .focus
        .values()
        .map(|record| record.assembly_bytes)
        .sum::<u64>();
    (state.bytes as u64)
        .saturating_add(staged)
        .saturating_add(reserved)
        .saturating_add(assembly)
}

fn physical_available(used: u64) -> u64 {
    (MAX_CACHE_BYTES as u64).saturating_sub(used)
}
