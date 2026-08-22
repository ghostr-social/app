use super::{
    objects, CacheState, CachedHlsGeneration, FocusRecord, SegmentedCache, SegmentedSnapshot,
};
use ghostr_engine::PostId;

impl SegmentedCache {
    pub fn invalidate_generation(&self, url: &str, generation: CachedHlsGeneration) -> bool {
        let mut state = self.lock();
        let Some(key) = objects::resolve_key(&state, url) else {
            return false;
        };
        if state.objects.get(&key).map(|object| object.generation()) != Some(generation) {
            return false;
        }
        let affected = invalidate_references(&mut state, &key);
        let changed = !affected.is_empty();
        for invalidated in affected {
            if !state.invalidated.contains(&invalidated) {
                state.invalidated.push(invalidated);
            }
        }
        objects::retain_referenced(&mut state);
        drop(state);
        if changed {
            self.changed.notify_waiters();
            self.invalidations
                .send_modify(|revision| *revision = revision.wrapping_add(1));
        }
        changed
    }

    pub(crate) fn take_invalidated(&self) -> Vec<(PostId, u64)> {
        std::mem::take(&mut self.lock().invalidated)
    }
}

fn invalidate_references(state: &mut CacheState, key: &str) -> Vec<(PostId, u64)> {
    state
        .focus
        .iter_mut()
        .filter_map(|(post, record)| {
            record.objects.iter().any(|object| object == key).then(|| {
                let invalidated = (post.clone(), record.generation);
                invalidate(record);
                invalidated
            })
        })
        .collect()
}

fn invalidate(record: &mut FocusRecord) {
    record.root_source = None;
    record.objects.clear();
    record.staged.clear();
    record.reserved_bytes = 0;
    record.assembly_bytes = 0;
    record.snapshot = SegmentedSnapshot::default();
}
