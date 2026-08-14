use super::{FeedId, FeedStore};
use crate::content::parsing::ParsedVideoPost;
use crate::content::social_graph::SocialGraph;

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
        open.add_occurrences(vec![fetched], graph)
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
        open.add_occurrences(fetched, graph)
    }
}
