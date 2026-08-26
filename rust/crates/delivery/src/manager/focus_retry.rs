use crate::manager::DeliveryWorker;
use ghostr_engine::PostId;

impl DeliveryWorker {
    pub(super) fn reset_focus_representations(
        &mut self,
        progressive: Vec<PostId>,
        hls: Vec<PostId>,
        cooldown_resets: &[PostId],
    ) -> Vec<PostId> {
        for post in progressive {
            self.cancel_transform(&post);
            self.cooldown_timers.cancel(&post);
            self.probes.representation_changed(&post);
            self.retry.representation_changed(&post);
        }
        for post in hls {
            let roots = self.segmented.tracked_roots(&post);
            self.probes.representation_changed(&post);
            self.retry.reconcile_hls_sources(&post, &roots);
        }
        self.reset_hls_cooldowns(cooldown_resets)
    }

    fn reset_hls_cooldowns(&mut self, posts: &[PostId]) -> Vec<PostId> {
        let restarting = posts
            .iter()
            .filter(|post| self.retry.is_cooling(post))
            .cloned()
            .collect();
        for post in posts {
            self.cooldown_timers.cancel(post);
            self.retry.cancel_hls_cooldown(post);
        }
        restarting
    }

    pub(super) fn apply_retry_focus_change(
        &mut self,
        previous: Option<&PostId>,
        current: Option<&PostId>,
    ) {
        self.retry.focus_changed(previous, current);
        if previous != current {
            if let Some(current) = current {
                if !self.retry.is_cooling(current) {
                    self.cooldown_timers.cancel(current);
                }
            }
        }
    }
}
