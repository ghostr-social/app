use super::context::{DurationBucket, WatchContext, WatchKey};
use super::stats::HazardStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const LONG_HALF_LIFE_MS: u64 = 14 * 24 * 60 * 60 * 1_000;
const SESSION_HALF_LIFE_MS: u64 = 2 * 60 * 1_000;
const RUNTIME_GROUPS: usize = 258;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) enum GroupKey {
    Global,
    User(WatchKey),
    Duration(DurationBucket),
    Category(WatchKey),
    Creator(WatchKey),
    Video(WatchKey),
    Session,
}

impl GroupKey {
    fn persistent(&self) -> bool {
        !matches!(self, Self::Session)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct GroupState {
    pub(super) key: GroupKey,
    pub(super) stats: HazardStats,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct WatchHierarchy {
    groups: HashMap<GroupKey, HazardStats>,
}

impl WatchHierarchy {
    pub(super) fn observe(
        &mut self,
        context: &WatchContext,
        watched_ms: u64,
        event: bool,
        now_ms: u64,
    ) {
        for (key, _) in context_groups(context, true) {
            self.ensure_room(&key);
            self.groups.entry(key.clone()).or_default().observe(
                watched_ms,
                event,
                now_ms,
                half_life(&key),
            );
        }
    }

    pub(super) fn survival(&self, context: &WatchContext, at_ms: u64, now_ms: u64) -> f64 {
        let mut weighted = super::stats::cold_survival(at_ms);
        let mut total = 1.0;
        for (key, similarity) in context_groups(context, true) {
            let Some(stats) = self.groups.get(&key) else {
                continue;
            };
            let samples = stats.effective_samples(now_ms, half_life(&key));
            let weight = similarity * samples / (samples + 4.0);
            weighted += stats.survival(at_ms, now_ms, half_life(&key)) * weight;
            total += weight;
        }
        (weighted / total).clamp(0.0, 1.0)
    }

    pub(super) fn reset_session(&mut self) {
        self.groups.remove(&GroupKey::Session);
    }

    pub(super) fn persistent_state(&self) -> Vec<GroupState> {
        self.groups
            .iter()
            .filter(|(key, _)| key.persistent())
            .map(|(key, stats)| GroupState {
                key: key.clone(),
                stats: stats.clone(),
            })
            .collect()
    }

    pub(super) fn from_state(groups: Vec<GroupState>, limit: usize) -> Self {
        let mut model = Self::default();
        for group in groups
            .into_iter()
            .filter(|group| group.key.persistent())
            .take(limit)
        {
            model.groups.insert(group.key, group.stats.sanitize());
        }
        model
    }

    fn ensure_room(&mut self, incoming: &GroupKey) {
        if self.groups.contains_key(incoming) || self.groups.len() < RUNTIME_GROUPS {
            return;
        }
        let evicted = self
            .groups
            .iter()
            .filter(|(key, _)| !matches!(key, GroupKey::Global | GroupKey::Session))
            .min_by_key(|(_, stats)| stats.last_used_ms)
            .map(|(key, _)| key.clone());
        if let Some(key) = evicted {
            self.groups.remove(&key);
        }
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "hierarchy/test_support.rs"]
mod test_support;

fn context_groups(context: &WatchContext, session: bool) -> Vec<(GroupKey, f64)> {
    let mut groups = vec![(GroupKey::Global, 0.7)];
    if let Some(user) = &context.user {
        groups.push((GroupKey::User(user.clone()), 0.8));
    }
    let duration = DurationBucket::of(context.duration_ms);
    groups.push((GroupKey::Duration(duration), 0.8));
    groups.extend(
        duration
            .neighbors()
            .map(|bucket| (GroupKey::Duration(bucket), 0.2)),
    );
    let category_weight = 0.8 / context.categories.len().max(1) as f64;
    groups.extend(
        context
            .categories
            .iter()
            .cloned()
            .map(|key| (GroupKey::Category(key), category_weight)),
    );
    if let Some(creator) = &context.creator {
        groups.push((GroupKey::Creator(creator.clone()), 1.0));
    }
    groups.push((GroupKey::Video(context.video.clone()), 1.5));
    if session {
        groups.push((GroupKey::Session, 4.0));
    }
    groups
}

fn half_life(key: &GroupKey) -> u64 {
    match key {
        GroupKey::Session => SESSION_HALF_LIFE_MS,
        _ => LONG_HALF_LIFE_MS,
    }
}
