use super::DeliveryState;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{PostId, VideoMeta};
use std::collections::HashSet;

impl DeliveryState {
    pub(super) fn upsert_progressive(&mut self, post: PostId, meta: VideoMeta) {
        let binding = self.catalog.upsert(post, meta);
        self.pending_representations
            .retain(|pending| pending.post() != binding.post());
        self.pending_representations.push(binding);
    }

    pub(crate) fn take_representation_bindings(&mut self) -> Vec<RepresentationBinding> {
        std::mem::take(&mut self.pending_representations)
    }

    pub(crate) fn retained_posts(&self) -> HashSet<PostId> {
        self.candidate_posts().into_iter().collect()
    }

    pub(super) fn prune_scheduling_state(&mut self) {
        let retained = self.retained_posts();
        self.catalog.retain(|post| retained.contains(post));
        self.pending_representations
            .retain(|binding| retained.contains(binding.post()));
    }
}
