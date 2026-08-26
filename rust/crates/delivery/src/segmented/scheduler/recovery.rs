use super::completion::{failure_detail, CompletedObject};
use super::progress::Pending;
use super::SegmentedDelivery;
use crate::manager::failure::FailureClass;
use crate::segmented::fetch::FetchFailure;
use crate::segmented::scheduler::FailureDisposition;
use crate::segmented::SegmentedPhase;
use ghostr_engine::PostId;

#[cfg(test)]
#[path = "recovery/same_stage_attempt_test.rs"]
mod same_stage_attempt_test;

pub(crate) enum SegmentedRecovery {
    Succeeded { post: PostId, root: String },
    Retry(Box<SegmentedRetry>),
    None,
}

pub(crate) struct SegmentedRetry {
    post: PostId,
    pending: Pending,
    roots: Vec<String>,
    disposition: FailureDisposition,
    detail: String,
}

pub(crate) enum RecoveryAction {
    SameStage,
    RestartObject,
    RestartRoot(String),
    Retire,
    Terminal,
}

impl SegmentedRetry {
    pub(crate) fn post(&self) -> &PostId {
        &self.post
    }

    pub(crate) fn root(&self) -> &str {
        &self.pending.root_source
    }

    pub(crate) fn roots(&self) -> &[String] {
        &self.roots
    }

    pub(crate) const fn disposition(&self) -> FailureDisposition {
        self.disposition
    }
}

impl SegmentedDelivery {
    pub(super) fn recovery(
        &self,
        post: &PostId,
        pending: &Pending,
        result: &Result<CompletedObject, FetchFailure>,
    ) -> SegmentedRecovery {
        let Some(roots) = self.current_roots(post, pending) else {
            return SegmentedRecovery::None;
        };
        match result {
            Ok(_) => self.success_recovery(post, pending),
            Err(error) if error.is_cancelled() || error.is_superseded() => SegmentedRecovery::None,
            Err(error) => SegmentedRecovery::Retry(Box::new(SegmentedRetry {
                post: post.clone(),
                pending: pending.clone(),
                roots,
                disposition: failure_disposition(pending, error),
                detail: failure_detail(error.reason()),
            })),
        }
    }

    fn success_recovery(&self, post: &PostId, pending: &Pending) -> SegmentedRecovery {
        match self.cache.snapshot(post.as_str()).phase {
            SegmentedPhase::Ready => SegmentedRecovery::Succeeded {
                post: post.clone(),
                root: pending.root_source.clone(),
            },
            _ => SegmentedRecovery::None,
        }
    }

    pub(crate) fn apply_recovery(&mut self, retry: SegmentedRetry, action: RecoveryAction) -> bool {
        if !self.owns_retry(&retry) {
            return false;
        }
        match action {
            RecoveryAction::SameStage => self.retry_same_stage(retry),
            RecoveryAction::RestartObject => self.restart_object(retry),
            RecoveryAction::RestartRoot(root) => self.restart_root(retry, root),
            RecoveryAction::Retire | RecoveryAction::Terminal => self.mark_failed(retry),
        }
    }

    pub(crate) fn revive(&mut self, post: &PostId, root: String) -> bool {
        self.revive_root(post, root)
    }

    pub(crate) fn roots(&self, post: &PostId) -> Option<Vec<String>> {
        self.targets
            .iter()
            .find(|target| &target.post == post)
            .map(|target| target.sources.clone())
    }

    fn current_roots(&self, post: &PostId, pending: &Pending) -> Option<Vec<String>> {
        if self.cache.focus_generation(post) != Some(pending.generation) {
            return None;
        }
        let roots = self.roots(post)?;
        crate::segmented::source_key::contains(&roots, &pending.root_source).then_some(roots)
    }

    fn owns_retry(&self, retry: &SegmentedRetry) -> bool {
        !self.active.contains_key(&retry.post)
            && self
                .current_roots(&retry.post, &retry.pending)
                .is_some_and(|roots| {
                    crate::segmented::source_key::same_members(&roots, &retry.roots)
                })
    }

    fn retry_same_stage(&mut self, retry: SegmentedRetry) -> bool {
        if !self
            .cache
            .release_stage_attempt(&retry.post, retry.pending.generation)
        {
            return false;
        }
        let attempt = self.allocate_attempt();
        let pending = retry.pending.retry_attempt(attempt);
        self.pending.insert(retry.post, pending);
        true
    }

    fn restart_object(&mut self, retry: SegmentedRetry) -> bool {
        if !self.cache.restart_stage_object(
            &retry.post,
            retry.pending.generation,
            &retry.pending.url,
        ) {
            return false;
        }
        let attempt = self.allocate_attempt();
        let pending = retry.pending.restart_object(attempt);
        self.pending.insert(retry.post, pending);
        true
    }

    fn restart_root(&mut self, retry: SegmentedRetry, root: String) -> bool {
        let Some(index) = retry.roots.iter().position(|known| known == &root) else {
            return false;
        };
        if !self
            .cache
            .reset_stage_retry(&retry.post, retry.pending.generation)
        {
            return false;
        }
        let attempt = self.allocate_attempt();
        let pending = Pending::root(retry.pending.generation, attempt, index, root);
        self.pending.insert(retry.post, pending);
        true
    }

    fn mark_failed(&self, retry: SegmentedRetry) -> bool {
        if !self
            .cache
            .mark_stage_failed(&retry.post, retry.pending.generation, retry.detail)
        {
            return false;
        }
        true
    }
}

fn failure_disposition(pending: &Pending, error: &FetchFailure) -> FailureDisposition {
    match error.disposition() {
        FailureDisposition::RestartObject if pending.generation_restarts > 0 => {
            FailureDisposition::Retry(FailureClass::Permanent)
        }
        disposition => disposition,
    }
}
