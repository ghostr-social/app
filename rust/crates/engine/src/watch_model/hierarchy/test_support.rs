use super::*;

impl WatchHierarchy {
    pub(crate) fn session_observations(&self, now_ms: u64) -> u64 {
        self.groups.get(&GroupKey::Session).map_or(0, |stats| {
            stats
                .effective_samples(now_ms, SESSION_HALF_LIFE_MS)
                .round() as u64
        })
    }

    #[cfg(test)]
    pub(crate) fn persistent_count(&self) -> usize {
        self.groups.keys().filter(|key| key.persistent()).count()
    }
}
