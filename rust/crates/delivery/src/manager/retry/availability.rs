use super::{CooldownId, RetryBook};
use ghostr_engine::PostId;

impl RetryBook {
    pub(crate) fn cool_down_until(
        &mut self,
        post: PostId,
        eligible_at_ms: u64,
    ) -> Option<CooldownId> {
        self.cooldowns.begin(post, eligible_at_ms)
    }

    pub(crate) fn cool_down_hls_until(
        &mut self,
        post: PostId,
        eligible_at_ms: u64,
    ) -> Option<CooldownId> {
        self.cooldowns.begin_strict(post, eligible_at_ms)
    }

    pub(crate) fn warm_up(&mut self, post: &PostId, cooldown: CooldownId) -> bool {
        self.cooldowns.finish(post, cooldown)
    }

    pub(crate) fn expedite_demand(&mut self, post: &PostId, offset: u64) -> bool {
        self.cooldowns.expedite_demand(post, offset)
    }

    pub(crate) fn is_cooling(&self, post: &PostId) -> bool {
        self.cooldowns.is_active(post)
    }

    pub(crate) fn cancel_hls_cooldown(&mut self, post: &PostId) {
        self.cooldowns.representation_changed(post);
    }

    pub(crate) fn cooling_until(&self, post: &PostId) -> Option<u64> {
        self.cooldowns.eligible_at_ms(post)
    }
}

#[cfg(test)]
#[path = "availability_axiom_test.rs"]
pub(crate) mod axiom_test_support;
