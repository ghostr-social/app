use super::FeedState;
use crate::api::feed::progress::FeedProgress;
use crate::discovery::content::candidates::{CandidateAdmission, VideoCandidate};
use crate::discovery::feed::cursor::retrieval_cursor;
use crate::discovery::feed::store::FeedId;
use crate::discovery::retrieval_types::{FeedContext, PlanFailure, RetrievalPurpose};
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
    ) -> Vec<VideoCandidate> {
        let Some(feed) = self.feed_for(context) else {
            return Vec::new();
        };
        match result {
            Ok(events) => self.apply_events(feed, &events, purpose),
            Err(_) => {
                self.record_failure(feed);
                Vec::new()
            }
        }
    }

    pub(crate) fn apply_progress(
        &mut self,
        context: &FeedContext,
        event: Event,
    ) -> Option<VideoCandidate> {
        let feed = self.feed_for(context)?;
        let inspected = self.candidates.inspect(&event);
        if let Some(post) = inspected.post {
            self.store.ingest_progress(feed, post, &self.graph);
        }
        admitted(inspected.admission)
    }

    pub(super) fn ingest_page(&mut self, feed: FeedId, events: &[Event]) -> Vec<VideoCandidate> {
        for event in events {
            self.profiles.ingest(event);
        }
        let batch = self.candidates.inspect_all(events);
        let published = if self.feeds.get(&feed).is_some_and(|it| it.first_loaded) {
            self.store.ingest_older_page(feed, batch.posts, &self.graph)
        } else {
            self.store.ingest_first_page(feed, batch.posts, &self.graph);
            true
        };
        self.store
            .set_retrieval_cursor(feed, retrieval_cursor(events));
        self.mark(feed, FeedProgress::record_page);
        if !published {
            self.store.touch(feed);
        }
        batch.admitted
    }

    pub(super) fn ingest_head(&mut self, feed: FeedId, events: &[Event]) -> Vec<VideoCandidate> {
        for event in events {
            self.profiles.ingest(event);
        }
        let batch = self.candidates.inspect_all(events);
        let published = self.store.ingest_head_page(feed, batch.posts, &self.graph);
        self.mark(feed, FeedProgress::record_page);
        if !published {
            self.store.touch(feed);
        }
        batch.admitted
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

    fn apply_events(
        &mut self,
        feed: FeedId,
        events: &[Event],
        purpose: RetrievalPurpose,
    ) -> Vec<VideoCandidate> {
        if self.is_head_refresh(feed, purpose) {
            self.ingest_head(feed, events)
        } else {
            self.ingest_page(feed, events)
        }
    }
}

fn admitted(admission: CandidateAdmission) -> Option<VideoCandidate> {
    match admission {
        CandidateAdmission::Accepted(candidate) | CandidateAdmission::Replaced(candidate) => {
            Some(candidate)
        }
        CandidateAdmission::Duplicate | CandidateAdmission::Rejected => None,
    }
}
