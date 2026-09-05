use super::{DeliveryWorker, PlanningStoreState};
use ghostr_engine::PostId;
use std::collections::HashSet;

impl DeliveryWorker {
    /// The planning slice of the window, widened to the full roster
    /// only under storage pressure so eviction can weigh every stored
    /// post, not just the current neighbourhood.
    pub(super) fn collection_window(
        &self,
        capacity: &ghostr_partial_store::partial_range_store::capacity::CapacitySnapshot,
    ) -> Vec<PostId> {
        if capacity.used_bytes() >= capacity.limit_bytes().saturating_mul(9) / 10 {
            return self.state.window_posts();
        }
        self.state.planning_window_posts()
    }

    pub(super) async fn collect_stored(
        &self,
        window: &[PostId],
        timeline_posts: &HashSet<PostId>,
    ) -> PlanningStoreState {
        let mut stored = PlanningStoreState::default();
        for post in window {
            let Some(binding) = self.state.catalog().binding(post) else {
                continue;
            };
            if let Ok(snapshot) = self.ctx.store.media_snapshot(post.as_str()).await {
                if snapshot
                    .binding()
                    .is_some_and(|stored| stored == &binding || stored.derives_from(&binding))
                {
                    if let Some(transformed) = snapshot
                        .binding()
                        .filter(|stored| stored.derives_from(&binding))
                    {
                        stored.transformed.insert(post.clone(), transformed.clone());
                    }
                    stored.insert(post.clone(), snapshot, timeline_posts.contains(post));
                }
            }
        }
        stored
    }
}
