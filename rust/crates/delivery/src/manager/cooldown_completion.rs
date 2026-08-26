use crate::manager::retry::CooldownId;
use crate::manager::DeliveryWorker;
use ghostr_engine::PostId;

impl DeliveryWorker {
    pub(super) fn finish_cooldown(&mut self, post: &PostId, cooldown: CooldownId) {
        if !self.cooldown_timers.finish(post, cooldown) {
            return;
        }
        if !self.retry.warm_up(post, cooldown) {
            return;
        }
        self.revive_segmented(post);
    }
}
