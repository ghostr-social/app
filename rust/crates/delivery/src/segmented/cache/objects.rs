use super::{CacheState, CachedHlsObject, SegmentedPhase, SegmentedSnapshot};
use std::collections::HashSet;

pub(super) fn insert(state: &mut CacheState, key: String, object: CachedHlsObject) {
    if let Some(replaced) = state.objects.remove(&key) {
        state.bytes = state.bytes.saturating_sub(replaced.body.len());
    }
    state.aliases.retain(|_, canonical| canonical != &key);
    state
        .aliases
        .insert(object.final_url.to_string(), key.clone());
    state.bytes = state.bytes.saturating_add(object.body.len());
    state.order.retain(|known| known != &key);
    state.order.push_back(key.clone());
    state.objects.insert(key, object);
}

pub(super) fn reclaimable_ready_bytes(state: &CacheState) -> u64 {
    let protected = referenced_keys(state, true);
    referenced_keys(state, false)
        .difference(&protected)
        .filter_map(|key| state.objects.get(key))
        .map(|object| object.body.len() as u64)
        .sum()
}

pub(super) fn reclaim_unprotected_ready(state: &mut CacheState) {
    for record in state
        .focus
        .values_mut()
        .filter(|record| !record.protected && reclaimable(record))
    {
        record.snapshot = SegmentedSnapshot::default();
        record.objects.clear();
    }
    retain_referenced(state);
}

pub(super) fn retain_referenced(state: &mut CacheState) {
    let retained = state
        .focus
        .values()
        .flat_map(|record| record.objects.iter().cloned())
        .collect::<HashSet<_>>();
    state.objects.retain(|key, _| retained.contains(key));
    state.aliases.retain(|_, key| retained.contains(key));
    state.order.retain(|key| retained.contains(key));
    state.bytes = state.objects.values().map(|object| object.body.len()).sum();
}

fn referenced_keys(state: &CacheState, protected: bool) -> HashSet<String> {
    state
        .focus
        .values()
        .filter(|record| record.protected == protected && reclaimable(record))
        .flat_map(|record| record.objects.iter().cloned())
        .collect()
}

fn reclaimable(record: &super::FocusRecord) -> bool {
    record.snapshot.phase == SegmentedPhase::Ready
        && record.staged.is_empty()
        && record.reserved_bytes == 0
}
