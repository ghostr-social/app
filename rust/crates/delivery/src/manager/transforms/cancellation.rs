use super::TransformJobs;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ActionId, PostId};
use std::collections::HashSet;

impl TransformJobs {
    pub(super) fn cancel_post(&mut self, post: &PostId) -> usize {
        let obsolete = self
            .active
            .iter()
            .filter(|(_, job)| &job.post == post)
            .map(|(action, _)| *action)
            .collect();
        self.cancel(obsolete)
    }

    pub(super) fn cancel_obsolete(&mut self, binding: &RepresentationBinding) -> usize {
        let obsolete = self
            .active
            .iter()
            .filter(|(_, job)| job.post == *binding.post() && job.binding != *binding)
            .map(|(action, _)| *action)
            .collect();
        self.cancel(obsolete)
    }

    pub(super) fn retain(&mut self, posts: &HashSet<PostId>) -> usize {
        let removed = self
            .active
            .iter()
            .filter(|(_, job)| !posts.contains(&job.post))
            .map(|(action, _)| *action)
            .collect();
        self.cancel(removed)
    }

    pub(super) fn clear(&mut self) -> usize {
        let removed = self.active.keys().copied().collect();
        self.cancel(removed)
    }

    fn cancel(&mut self, actions: Vec<ActionId>) -> usize {
        actions
            .into_iter()
            .filter(|action| self.cancel_one(*action))
            .count()
    }

    fn cancel_one(&mut self, action: ActionId) -> bool {
        let Some(job) = self.active.get_mut(&action) else {
            return false;
        };
        if job.cancellation_requested || !job.control.cancel() {
            return false;
        }
        job.cancellation_requested = true;
        true
    }
}
