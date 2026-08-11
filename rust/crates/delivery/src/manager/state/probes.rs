use super::DeliveryState;
use ghostr_engine::PostId;

impl DeliveryState {
    /// Origin IO follows only the startup-critical attention prefix.
    /// Before explicit focus, only the newest projected candidate is local.
    pub(crate) fn protected_posts(&self) -> Vec<PostId> {
        let Some(current) = self.focus.current() else {
            return Vec::new();
        };
        let start = self
            .focus
            .window()
            .iter()
            .position(|post| std::ptr::eq(post, current))
            .unwrap_or_default();
        let limit = if self.projection_focus {
            1
        } else {
            self.effective.startable_target.max(1)
        };
        self.focus.window()[start..]
            .iter()
            .take(limit)
            .cloned()
            .collect()
    }

    pub(crate) fn probe_posts(&self) -> Vec<PostId> {
        self.protected_posts()
    }
}
