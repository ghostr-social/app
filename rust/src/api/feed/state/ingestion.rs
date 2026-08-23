use super::FeedState;
use crate::api::feed::progress::FeedProgress;
use crate::discovery::content::candidates::{CandidateAdmission, VideoCandidate};
use crate::discovery::content::deletions::deletion_claims;
use crate::discovery::content::reposts::{GENERIC_REPOST_KIND, REPOST_KIND};
use crate::discovery::feed::spec::FeedSpec;
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
        cursor: Option<nostr_sdk::Timestamp>,
        purpose: RetrievalPurpose,
        complete: bool,
    ) -> Vec<VideoCandidate> {
        let Some(feed) = self.feed_for(context) else {
            return Vec::new();
        };
        match result {
            Ok(events) if complete => self.apply_events(feed, &events, cursor, purpose),
            Ok(events) => self.apply_partial_events(feed, &events),
            Err(_) => {
                self.record_failure(feed);
                Vec::new()
            }
        }
    }

    fn apply_partial_events(&mut self, feed: FeedId, events: &[Event]) -> Vec<VideoCandidate> {
        for event in events {
            self.profiles.ingest(event);
        }
        let batch = self.candidates.inspect_all(events);
        self.store.ingest_head_page(feed, batch.posts, &self.graph);
        self.ingest_deletion_events(feed, events);
        self.record_failure(feed);
        batch.admitted
    }

    pub(crate) fn apply_progress(
        &mut self,
        context: &FeedContext,
        event: Event,
    ) -> Option<VideoCandidate> {
        let feed = self.feed_for(context)?;
        self.ingest_deletion_events(feed, std::slice::from_ref(&event));
        let following = matches!(self.store.spec(feed), FeedSpec::Following { .. });
        if following || waits_for_deletion_checks(&event) {
            return None;
        }
        let inspected = self.candidates.inspect(&event);
        if let Some(post) = inspected.post {
            self.store.ingest_progress(feed, post, &self.graph);
        }
        admitted(inspected.admission)
    }

    pub(super) fn ingest_page(
        &mut self,
        feed: FeedId,
        events: &[Event],
        cursor: Option<nostr_sdk::Timestamp>,
    ) -> Vec<VideoCandidate> {
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
        self.ingest_deletion_events(feed, events);
        self.store.set_retrieval_cursor(feed, cursor);
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
        let retracted = self.ingest_deletion_events(feed, events);
        self.mark(feed, FeedProgress::record_page);
        if !published && !retracted {
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
        cursor: Option<nostr_sdk::Timestamp>,
        purpose: RetrievalPurpose,
    ) -> Vec<VideoCandidate> {
        if self.is_head_refresh(feed, purpose) {
            self.ingest_head(feed, events)
        } else {
            self.ingest_page(feed, events, cursor)
        }
    }

    fn ingest_deletion_events(&mut self, feed: FeedId, events: &[Event]) -> bool {
        self.store
            .ingest_deletions(feed, deletion_claims(events), &self.graph)
    }
}

fn waits_for_deletion_checks(event: &Event) -> bool {
    [REPOST_KIND, GENERIC_REPOST_KIND].contains(&event.kind.as_u16())
}

fn admitted(admission: CandidateAdmission) -> Option<VideoCandidate> {
    match admission {
        CandidateAdmission::Accepted(candidate) | CandidateAdmission::Replaced(candidate) => {
            Some(candidate)
        }
        CandidateAdmission::Duplicate | CandidateAdmission::Rejected => None,
    }
}
