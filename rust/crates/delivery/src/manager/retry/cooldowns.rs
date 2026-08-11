use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet, VecDeque};

const DEMAND_MEMORY_LIMIT: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CooldownId(u64);

#[derive(Default)]
pub(super) struct Cooldowns {
    active: HashMap<PostId, CooldownId>,
    demanded_offsets: HashMap<PostId, VecDeque<u64>>,
    credits: HashSet<PostId>,
    sequence: u64,
}

impl Cooldowns {
    pub(super) fn begin(&mut self, post: PostId) -> Option<CooldownId> {
        if self.active.contains_key(&post) {
            return None;
        }
        if self.credits.remove(&post) {
            return None;
        }
        let cooldown = CooldownId(self.sequence);
        self.sequence = self.sequence.wrapping_add(1);
        self.active.insert(post, cooldown);
        Some(cooldown)
    }

    pub(super) fn focus_changed(&mut self, previous: Option<&PostId>, current: Option<&PostId>) {
        if previous == current {
            return;
        }
        if let Some(previous) = previous {
            self.credits.remove(previous);
            self.demanded_offsets.remove(previous);
        }
        if let Some(current) = current {
            self.expedite(current);
        }
    }

    pub(super) fn finish(&mut self, post: &PostId, cooldown: CooldownId) -> bool {
        if self.active.get(post) != Some(&cooldown) {
            return false;
        }
        self.active.remove(post);
        true
    }

    pub(super) fn expedite_demand(&mut self, post: &PostId, offset: u64) -> bool {
        let offsets = self.demanded_offsets.entry(post.clone()).or_default();
        if offsets.contains(&offset) {
            return false;
        }
        if offsets.len() == DEMAND_MEMORY_LIMIT {
            offsets.pop_front();
        }
        offsets.push_back(offset);
        self.expedite(post);
        true
    }

    pub(super) fn representation_changed(&mut self, post: &PostId) {
        self.active.remove(post);
        self.reset_demand(post);
    }

    pub(super) fn note_success(&mut self, post: &PostId) {
        self.active.remove(post);
        self.reset_demand(post);
    }

    pub(super) fn is_active(&self, post: &PostId) -> bool {
        self.active.contains_key(post)
    }

    pub(super) fn clear(&mut self) {
        self.active.clear();
        self.demanded_offsets.clear();
        self.credits.clear();
    }

    pub(super) fn retain_demand(&mut self, retained: &HashSet<PostId>) {
        self.active.retain(|post, _| retained.contains(post));
        self.demanded_offsets
            .retain(|post, _| retained.contains(post));
        self.credits.retain(|post| retained.contains(post));
    }

    pub(super) fn clear_credit(&mut self, post: &PostId) {
        self.credits.remove(post);
    }

    fn expedite(&mut self, post: &PostId) {
        if self.active.remove(post).is_none() {
            self.credits.insert(post.clone());
        }
    }

    fn reset_demand(&mut self, post: &PostId) {
        self.demanded_offsets.remove(post);
        self.credits.remove(post);
    }

    #[cfg(test)]
    pub(super) fn demand_tracking_units(&self) -> usize {
        self.demanded_offsets.values().map(VecDeque::len).sum()
    }
}
