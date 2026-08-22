use super::DeliveryState;
use crate::transform::TransformProfile;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use std::collections::HashMap;

impl DeliveryState {
    pub(crate) fn configure_transform(&mut self, profile: Option<TransformProfile>) {
        self.transform_profile = profile;
    }

    pub(crate) const fn transform_profile(&self) -> Option<TransformProfile> {
        self.transform_profile
    }

    pub(crate) fn transform_available_for(&self, post: &PostId) -> Option<TransformProfile> {
        if !self.active_transforms.is_empty() || self.transformed_posts.contains_key(post) {
            return None;
        }
        self.transform_profile
    }

    pub(crate) fn begin_transform(&mut self, post: PostId) -> bool {
        if !self.active_transforms.is_empty() {
            return false;
        }
        self.active_transforms.insert(post)
    }

    pub(crate) fn finish_transform(&mut self, post: &PostId) {
        self.active_transforms.remove(post);
    }

    pub(crate) fn replace_transformed_posts(
        &mut self,
        posts: HashMap<PostId, RepresentationBinding>,
    ) {
        self.transformed_posts = posts;
    }

    pub(crate) fn playback_binding(&self, post: &PostId) -> Option<RepresentationBinding> {
        self.transformed_posts
            .get(post)
            .cloned()
            .or_else(|| self.catalog().binding(post))
    }
}
