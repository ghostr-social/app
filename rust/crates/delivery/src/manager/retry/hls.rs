use super::{Retry, RetryBook, Source};
use crate::manager::failure::FailureClass;
use core::time::Duration;
use ghostr_engine::PostId;
use std::collections::HashSet;
use tokio::time::Instant;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HlsRootAvailability {
    Live(Vec<String>),
    Waiting(Duration),
    Empty,
}

impl RetryBook {
    pub(crate) fn reconcile_hls_sources(&mut self, post: &PostId, roots: &[String]) {
        let retained: HashSet<_> = roots
            .iter()
            .map(|root| Source::new(post.clone(), root))
            .collect();
        self.attempts
            .retain(|source, _| &source.post != post || retained.contains(source));
        self.retired
            .retain(|source, _| &source.post != post || retained.contains(source));
    }

    pub(crate) fn note_hls_failure(&mut self, source: Source, class: FailureClass) -> Retry {
        if self.is_retired(&source) {
            return Retry::GiveUp;
        }
        let attempts = self.charge(&source);
        if attempts >= self.budget(class) {
            self.retire(source, class, true)
        } else {
            Retry::After(self.policy.backoff(attempts))
        }
    }

    pub(crate) fn hls_root_availability(
        &self,
        post: &PostId,
        roots: &[String],
    ) -> HlsRootAvailability {
        if roots.is_empty() {
            return HlsRootAvailability::Empty;
        }
        let now = Instant::now();
        let mut live: Vec<_> = roots
            .iter()
            .filter(|root| self.retirement_wait(post, root, now).is_none())
            .cloned()
            .collect();
        live.sort_by_key(|root| self.failure_count(post, root));
        if !live.is_empty() {
            return HlsRootAvailability::Live(live);
        }
        let wait = roots
            .iter()
            .filter_map(|root| self.retirement_wait(post, root, now))
            .min()
            .unwrap_or_default();
        HlsRootAvailability::Waiting(wait)
    }

    pub(crate) fn preferred_hls_alternative(
        &self,
        post: &PostId,
        failed: &str,
        roots: &[String],
    ) -> Option<String> {
        let failed = Source::new(post.clone(), failed);
        let failed_count = self.attempts.get(&failed).copied().unwrap_or(0);
        self.live_urls(post, roots).into_iter().find(|root| {
            let source = Source::new(post.clone(), root);
            source != failed && self.failure_count(post, root) < failed_count
        })
    }

    fn retirement_wait(&self, post: &PostId, root: &str, now: Instant) -> Option<Duration> {
        self.retired
            .get(&Source::new(post.clone(), root))
            .filter(|retirement| retirement.until > now)
            .map(|retirement| retirement.until.duration_since(now))
    }
}
