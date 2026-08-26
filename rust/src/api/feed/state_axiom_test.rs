use super::*;

use crate::discovery::retrieval_types::PlanFailure;

use nostr_sdk::Event;

impl FeedState {
    /// Compatibility helper for focused state tests.
    pub(crate) fn apply(&mut self, context: &FeedContext, result: Result<Vec<Event>, PlanFailure>) {
        let Some(feed) = self.feed_for(context) else {
            return;
        };
        match result {
            Ok(events) => {
                let cursor = crate::discovery::feed::cursor::retrieval_cursor(&events);
                self.ingest_page(feed, &events, cursor);
            }
            Err(_) => self.record_failure(feed),
        }
    }
}
