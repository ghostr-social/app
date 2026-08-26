use crate::manager::retry::{HlsRootAvailability, Retry, Source};
use crate::manager::{time, DeliveryWorker};
use crate::segmented::scheduler::{
    FailureDisposition, RecoveryAction, SegmentedDone, SegmentedFinish, SegmentedRecovery,
    SegmentedRetry,
};
use ghostr_engine::{ActionId, PostId};

impl DeliveryWorker {
    pub(super) fn finish_segmented(&mut self, done: SegmentedDone) {
        let Some(finish) = self.segmented.finish(done) else {
            return;
        };
        let recovery = self.settle_segmented(finish);
        self.apply_segmented_recovery(recovery);
        self.request_immediate_replan();
    }

    fn settle_segmented(&mut self, finish: SegmentedFinish) -> SegmentedRecovery {
        let observed_at_ms = time::unix_time_ms();
        let actual_network_bytes = finish
            .actual_resources
            .map_or(0, |actual| actual.network_bytes);
        self.warp_planner.reconcile_network_reservation(
            finish.resources.reserved_network_bytes(),
            actual_network_bytes,
            observed_at_ms,
        );
        if let Some(observation) = finish.observation {
            self.keeper.note_hls(&observation);
        }
        self.resolve_segmented(finish.action, finish.outcome, finish.actual_resources);
        finish.recovery
    }

    fn resolve_segmented(
        &self,
        action: ActionId,
        outcome: ghostr_engine::adaptive::DecisionOutcome,
        actual: Option<ghostr_engine::adaptive::ResourceCost>,
    ) {
        let observed_at_ms = time::unix_time_ms();
        match actual {
            Some(resources) => self.commands.resolve_decision_with_resources(
                action,
                outcome,
                resources,
                observed_at_ms,
            ),
            None => self
                .commands
                .resolve_decision(action, outcome, observed_at_ms),
        };
    }

    fn apply_segmented_recovery(&mut self, recovery: SegmentedRecovery) {
        match recovery {
            SegmentedRecovery::Succeeded { post, root } => {
                self.note_successful_attempt(&post, &root);
            }
            SegmentedRecovery::Retry(retry) => self.retry_segmented(*retry),
            SegmentedRecovery::None => {}
        }
    }

    fn retry_segmented(&mut self, retry: SegmentedRetry) {
        let class = match retry.disposition() {
            FailureDisposition::Terminal => {
                self.segmented
                    .apply_recovery(retry, RecoveryAction::Terminal);
                return;
            }
            FailureDisposition::Requeue => {
                self.segmented
                    .apply_recovery(retry, RecoveryAction::SameStage);
                return;
            }
            FailureDisposition::RestartObject => {
                self.segmented
                    .apply_recovery(retry, RecoveryAction::RestartObject);
                return;
            }
            FailureDisposition::Retry(class) => class,
        };
        let post = retry.post().clone();
        let root = retry.root().to_owned();
        match self
            .retry
            .note_hls_failure(Source::new(post.clone(), &root), class)
        {
            Retry::After(wait) => self.retry_segmented_after(retry, post, wait),
            Retry::GiveUp => self.fallback_or_retire(retry),
        }
    }

    fn retry_segmented_after(
        &mut self,
        retry: SegmentedRetry,
        post: PostId,
        wait: core::time::Duration,
    ) {
        if let Some(root) = self
            .retry
            .preferred_hls_alternative(&post, retry.root(), retry.roots())
        {
            self.segmented
                .apply_recovery(retry, RecoveryAction::RestartRoot(root));
            return;
        }
        if self
            .segmented
            .apply_recovery(retry, RecoveryAction::SameStage)
        {
            self.start_hls_cooldown(post, wait);
        }
    }

    fn fallback_or_retire(&mut self, retry: SegmentedRetry) {
        let post = retry.post().clone();
        let roots = retry.roots().to_vec();
        match self.retry.hls_root_availability(&post, &roots) {
            HlsRootAvailability::Live(live) => {
                self.segmented
                    .apply_recovery(retry, RecoveryAction::RestartRoot(live[0].clone()));
            }
            HlsRootAvailability::Waiting(wait) => self.retire_hls(retry, post, Some(wait)),
            HlsRootAvailability::Empty => self.retire_hls(retry, post, None),
        }
    }

    pub(super) fn revive_segmented(&mut self, post: &PostId) {
        let Some(roots) = self.segmented.roots(post) else {
            return;
        };
        match self.retry.hls_root_availability(post, &roots) {
            HlsRootAvailability::Live(live) => {
                self.segmented.revive(post, live[0].clone());
            }
            HlsRootAvailability::Waiting(wait) => {
                self.start_hls_cooldown(post.clone(), wait);
            }
            HlsRootAvailability::Empty => {}
        }
    }

    pub(super) fn reconcile_segmented_roots(&mut self) {
        for (post, roots) in self.segmented.root_sets() {
            match self.retry.hls_root_availability(&post, &roots) {
                HlsRootAvailability::Live(live) => {
                    self.segmented.select_pending_root(&post, &live[0]);
                }
                HlsRootAvailability::Waiting(wait) => {
                    if self.segmented.suspend_pending_roots(&post) {
                        self.start_hls_cooldown(post, wait);
                    }
                }
                HlsRootAvailability::Empty => {}
            }
        }
    }

    pub(super) fn restart_segmented_roots(&mut self, posts: &[PostId]) {
        for post in posts {
            let Some(roots) = self.segmented.roots(post) else {
                continue;
            };
            if let HlsRootAvailability::Live(live) = self.retry.hls_root_availability(post, &roots)
            {
                self.segmented.restart_pending_root(post, &live[0]);
            }
        }
    }

    fn retire_hls(
        &mut self,
        retry: SegmentedRetry,
        post: PostId,
        wait: Option<core::time::Duration>,
    ) {
        if self.segmented.apply_recovery(retry, RecoveryAction::Retire) {
            if let Some(wait) = wait {
                self.start_hls_cooldown(post, wait);
            }
        }
    }
}
