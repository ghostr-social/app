use super::{FeedId, FeedStore};
use crate::event_parsing::ParsedVideoPost;
use crate::feed_assembly::select_posts;
use crate::social_graph::SocialGraph;

impl FeedStore {
    pub fn begin_background_load(&mut self, feed: FeedId) {
        if let Some(open) = self.feeds.get_mut(&feed) {
            open.in_flight = true;
        }
    }

    /// Merges one relay event while its retrieval remains in flight.
    /// Full canonical selection keeps arrival order from changing the feed.
    pub fn ingest_progress(
        &mut self,
        feed: FeedId,
        fetched: ParsedVideoPost,
        graph: &SocialGraph,
    ) -> bool {
        let Some(open) = self.feeds.get_mut(&feed) else {
            return false;
        };
        let before = open.posts.clone();
        let mut combined = std::mem::take(&mut open.posts);
        combined.push(fetched);
        open.posts = select_posts(&open.spec, combined, graph);
        open.trim();
        if open.posts == before {
            return false;
        }
        open.notify();
        true
    }

    /// Reconciles one full head refresh without moving the historical cursor.
    pub fn ingest_head_page(
        &mut self,
        feed: FeedId,
        fetched: Vec<ParsedVideoPost>,
        graph: &SocialGraph,
    ) -> bool {
        let Some(open) = self.feeds.get_mut(&feed) else {
            return false;
        };
        open.in_flight = false;
        let before = open.posts.clone();
        let mut combined = std::mem::take(&mut open.posts);
        combined.extend(fetched);
        open.posts = select_posts(&open.spec, combined, graph);
        open.trim();
        let changed = open.posts != before;
        if changed {
            open.notify();
        }
        changed
    }
}
