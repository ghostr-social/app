use super::FeedState;
use crate::api::feed_progress::FeedProgress;
use crate::discovery::discovery_scheduler::RetrievalPurpose;
use crate::discovery::event_parsing::{video_post_from_event, ParsedVideoPost};
use crate::discovery::feed_cursor::retrieval_cursor;
use crate::discovery::feed_store::FeedId;
use crate::discovery::plan_executor::PlanFailure;
use crate::discovery::retrieval_queue::FeedContext;
use nostr_sdk::Event;

impl FeedState {
    pub(crate) fn apply_started(&mut self, context: &FeedContext) {
        let Some(feed) = self.feed_for(context) else {
            return;
        };
        self.store.begin_background_load(feed);
        self.mark(feed, FeedProgress::await_more);
        self.store.touch(feed);
    }

    pub(crate) fn apply_retrieval(
        &mut self,
        context: &FeedContext,
        result: Result<Vec<Event>, PlanFailure>,
        purpose: RetrievalPurpose,
    ) {
        let Some(feed) = self.feed_for(context) else {
            return;
        };
        match result {
            Ok(events) if self.is_head_refresh(feed, purpose) => self.ingest_head(feed, &events),
            Ok(events) => self.ingest_page(feed, &events),
            Err(_) => self.record_failure(feed),
        }
    }

    pub(crate) fn apply_progress(&mut self, context: &FeedContext, event: Event) {
        let Some(feed) = self.feed_for(context) else {
            return;
        };
        let Some(post) = video_post_from_event(&event) else {
            return;
        };
        self.store.ingest_progress(feed, post, &self.graph);
    }

    pub(super) fn ingest_page(&mut self, feed: FeedId, events: &[Event]) {
        for event in events {
            self.profiles.ingest(event);
        }
        let posts = parsed_posts(events);
        let published = if self.feeds.get(&feed).is_some_and(|it| it.first_loaded) {
            self.store.ingest_older_page(feed, posts, &self.graph)
        } else {
            self.store.ingest_first_page(feed, posts, &self.graph);
            true
        };
        self.store
            .set_retrieval_cursor(feed, retrieval_cursor(events));
        self.mark(feed, FeedProgress::record_page);
        if !published {
            self.store.touch(feed);
        }
    }

    pub(super) fn ingest_head(&mut self, feed: FeedId, events: &[Event]) {
        for event in events {
            self.profiles.ingest(event);
        }
        let published = self
            .store
            .ingest_head_page(feed, parsed_posts(events), &self.graph);
        self.mark(feed, FeedProgress::record_page);
        if !published {
            self.store.touch(feed);
        }
    }

    pub(super) fn record_failure(&mut self, feed: FeedId) {
        self.store.fail_load_more(feed);
        self.mark(feed, FeedProgress::record_failure);
        self.store.touch(feed);
    }

    fn is_head_refresh(&self, feed: FeedId, purpose: RetrievalPurpose) -> bool {
        purpose == RetrievalPurpose::Head
            && self
                .feeds
                .get(&feed)
                .is_some_and(|progress| progress.first_loaded)
    }
}

fn parsed_posts(events: &[Event]) -> Vec<ParsedVideoPost> {
    events.iter().filter_map(video_post_from_event).collect()
}
