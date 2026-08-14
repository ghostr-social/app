use super::{FeedId, FeedStore};
use crate::content::parsing::ParsedVideoPost;
use crate::content::social_graph::SocialGraph;
use crate::feed::store_cursor::{older_cursor, post_cursor};
use nostr_sdk::Timestamp;

impl FeedStore {
    pub fn ingest_first_page(
        &mut self,
        feed: FeedId,
        fetched: Vec<ParsedVideoPost>,
        graph: &SocialGraph,
    ) {
        let Some(open) = self.feeds.get_mut(&feed) else {
            return;
        };
        open.occurrences = fetched;
        open.compact_occurrences();
        open.reproject(graph);
        open.cursor = post_cursor(&open.posts);
        open.in_flight = false;
        open.notify();
    }

    pub fn fail_load_more(&mut self, feed: FeedId) {
        if let Some(open) = self.feeds.get_mut(&feed) {
            open.in_flight = false;
        }
    }

    pub fn set_retrieval_cursor(&mut self, feed: FeedId, cursor: Option<Timestamp>) {
        if let (Some(open), Some(cursor)) = (self.feeds.get_mut(&feed), cursor) {
            open.cursor = Some(cursor);
        }
    }

    pub fn ingest_older_page(
        &mut self,
        feed: FeedId,
        fetched: Vec<ParsedVideoPost>,
        graph: &SocialGraph,
    ) -> bool {
        let Some(open) = self.feeds.get_mut(&feed) else {
            return false;
        };
        open.in_flight = false;
        open.cursor = older_cursor(&open.spec, open.cursor, &fetched);
        open.add_occurrences(fetched, graph)
    }
}
