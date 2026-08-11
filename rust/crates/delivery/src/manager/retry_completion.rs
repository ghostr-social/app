//! Applying retry decisions after transfer and probe failures.

use crate::manager::failure::FailureClass;
use crate::manager::retry::{Retry, Source};
use crate::manager::DeliveryWorker;
use ghostr_engine::PostId;
use log::warn;
use std::time::Duration;

impl DeliveryWorker {
    /// Charges one failed attempt to the source: either a paced retry
    /// or immediate fallback to an available mirror.
    pub(crate) fn note_failed_attempt(&mut self, post: &PostId, url: &str, class: FailureClass) {
        let retry = self
            .retry
            .note_failure(Source::new(post.clone(), url.to_owned()), class);
        match retry {
            Retry::After(_) if self.has_ready_alternative(post, url) => {}
            Retry::After(wait) => self.start_cooldown(post.clone(), wait),
            Retry::GiveUp => self.retire_source(post, url),
        }
    }

    fn has_ready_alternative(&self, post: &PostId, failed: &str) -> bool {
        self.state.catalog().lookup(post).is_some_and(|entry| {
            self.retry
                .has_ready_alternative(post, failed, &entry.meta.urls)
        })
    }

    fn retire_source(&mut self, post: &PostId, url: &str) {
        let id = post.as_str();
        if self.is_servable(post) {
            warn!("Giving up on {url} for {id}; another source remains");
            return;
        }
        warn!("No working source left for {id}; reporting it unplayable");
    }

    pub(crate) fn start_cooldown(&mut self, post: PostId, wait: Duration) {
        let Some(cooldown) = self.retry.cool_down(post.clone()) else {
            return;
        };
        self.cooldown_timers
            .start(post, cooldown, wait, self.ctx.events.clone());
    }

    pub(crate) fn expedite_demand(&mut self, post: &PostId, offset: u64) -> bool {
        let accepted = self.retry.expedite_demand(post, offset);
        if accepted {
            self.cooldown_timers.cancel(post);
        }
        accepted
    }

    pub(crate) fn note_successful_attempt(&mut self, post: &PostId, url: &str) {
        self.cooldown_timers.cancel(post);
        self.retry
            .note_success(&Source::new(post.clone(), url.to_owned()));
    }
}
