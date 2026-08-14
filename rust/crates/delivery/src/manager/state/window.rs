use super::DeliveryState;
use ghostr_engine::PostId;
use std::collections::HashSet;

const PLANNING_WINDOW_BEHIND: usize = 3;
const PLANNING_WINDOW_AHEAD: usize = 24;

impl DeliveryState {
    /// Window posts in scroll order, deduplicated.
    pub(crate) fn window_posts(&self) -> Vec<PostId> {
        let mut seen = HashSet::new();
        self.focus
            .window()
            .iter()
            .filter(|post| seen.insert((*post).clone()))
            .cloned()
            .collect()
    }

    pub(crate) fn demand_posts(&self) -> HashSet<PostId> {
        let mut posts = HashSet::new();
        let Some(current) = self.focus.current() else {
            return posts;
        };
        posts.insert(current.clone());
        let next = self
            .focus
            .window()
            .iter()
            .position(|post| post == current)
            .and_then(|index| self.focus.window().get(index + 1));
        posts.extend(next.cloned());
        posts
    }

    pub(crate) fn planning_window_posts(&self) -> Vec<PostId> {
        let posts = self.window_posts();
        let Some(current) = self.focus.current() else {
            return posts;
        };
        let current = posts
            .iter()
            .position(|post| post == current)
            .unwrap_or_default();
        let start = current.saturating_sub(PLANNING_WINDOW_BEHIND);
        let end = current
            .saturating_add(PLANNING_WINDOW_AHEAD + 1)
            .min(posts.len());
        posts[start..end].to_vec()
    }

    pub(crate) fn candidate_posts(&self) -> Vec<PostId> {
        let mut posts = self.window_posts();
        let mut seen: HashSet<_> = posts.iter().cloned().collect();
        posts.extend(
            self.candidates
                .ranked()
                .into_iter()
                .filter(|post| seen.insert(post.clone())),
        );
        posts
    }
}
