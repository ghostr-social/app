use super::DeliveryState;
use ghostr_engine::PostId;

impl DeliveryState {
    /// Exposes every upcoming post to adaptive probe admission. Before
    /// explicit product focus, only the newest projected candidate is local.
    pub(crate) fn probe_posts(&self) -> Vec<PostId> {
        let Some(current) = self.focus.current() else {
            return Vec::new();
        };
        let start = self
            .focus
            .window()
            .iter()
            .position(|post| std::ptr::eq(post, current))
            .unwrap_or_default();
        let upcoming = &self.focus.window()[start..];
        match self.projection_focus {
            true => upcoming.first().cloned().into_iter().collect(),
            false => upcoming.to_vec(),
        }
    }
}
