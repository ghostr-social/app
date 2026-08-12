use super::DeliveryState;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{PostId, VideoMeta};
use std::collections::HashSet;

impl DeliveryState {
    pub(super) fn upsert_progressive(&mut self, post: PostId, meta: VideoMeta) {
        let previous = self.catalog.binding(&post);
        let binding = self.catalog.upsert(post, meta);
        self.accept_binding(previous, binding);
    }

    pub(super) fn upsert_progressive_renditions(
        &mut self,
        post: PostId,
        meta: VideoMeta,
        renditions: Vec<VideoRendition>,
    ) {
        let previous = self.catalog.binding(&post);
        let binding = self.catalog.upsert_with_renditions(post, meta, renditions);
        self.accept_binding(previous, binding);
    }

    fn accept_binding(
        &mut self,
        previous: Option<RepresentationBinding>,
        binding: RepresentationBinding,
    ) {
        if previous.as_ref() == Some(&binding) {
            return;
        }
        if previous.as_ref().is_some_and(|old| old != &binding) {
            self.forget_evictions(binding.post());
            self.changed_representations.push(binding.post().clone());
        }
        self.queue_representation(binding);
    }

    pub(crate) fn queue_representation(&mut self, binding: RepresentationBinding) {
        if self
            .pending_representations
            .iter()
            .any(|pending| pending == &binding)
        {
            return;
        }
        self.pending_representations
            .retain(|pending| pending.post() != binding.post());
        self.pending_representations.push(binding);
    }

    pub(crate) fn take_representation_bindings(&mut self) -> Vec<RepresentationBinding> {
        std::mem::take(&mut self.pending_representations)
    }

    pub(crate) fn take_changed_representations(&mut self) -> Vec<PostId> {
        std::mem::take(&mut self.changed_representations)
    }

    pub(crate) fn retained_posts(&self) -> HashSet<PostId> {
        self.candidate_posts().into_iter().collect()
    }

    pub(super) fn prune_scheduling_state(&mut self) {
        let retained = self.retained_posts();
        self.catalog.retain(|post| retained.contains(post));
        self.retain_evictions(&retained);
        self.pending_representations
            .retain(|binding| retained.contains(binding.post()));
        self.changed_representations
            .retain(|post| retained.contains(post));
    }
}
