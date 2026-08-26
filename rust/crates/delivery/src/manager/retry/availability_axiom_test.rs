use super::*;

impl RetryBook {
    /// Marks the post as pausing between attempts. `None` when a pause already owns a timer.
    pub(crate) fn cool_down(&mut self, post: PostId) -> Option<CooldownId> {
        self.cool_down_until(post, u64::MAX)
    }
}
