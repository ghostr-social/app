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
        let Some(index) = self.focus.window().iter().position(|post| post == current) else {
            return posts;
        };
        posts.extend(
            self.focus
                .window()
                .iter()
                .skip(index)
                .take(self.ready_target.saturating_add(1))
                .cloned(),
        );
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

    pub(crate) fn timeline_window_posts(&self) -> Vec<PostId> {
        let posts = self.planning_window_posts();
        let Some(current) = self.focus.current() else {
            return posts;
        };
        let Some(index) = posts.iter().position(|post| post == current) else {
            return posts;
        };
        posts[index..]
            .iter()
            .chain(posts[..index].iter().rev())
            .cloned()
            .collect()
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
