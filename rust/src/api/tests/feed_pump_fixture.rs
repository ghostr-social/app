use crate::api::feed::state::FeedState;
use crate::api::runtime::discovery::{lock, OutcomeSinks, SharedFeedState};
use crate::api::tests::feed_fixtures::video_note;
use crate::api::tests::outbox_runtime_support::test_bootstrap;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::feed::store::FeedId;
use crate::discovery::retrieval_types::{FeedContext, RetrievalOutcome, RetrievalPurpose};
use nostr_sdk::{Event, Keys};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};

pub(super) struct FeedPumpFixture {
    pub(super) state: SharedFeedState,
    pub(super) feed: FeedId,
    context: FeedContext,
    pub(super) revisions: watch::Receiver<u64>,
    pub(super) sender: mpsc::UnboundedSender<RetrievalOutcome>,
    pub(super) outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
}

impl FeedPumpFixture {
    pub(super) fn new() -> Self {
        let state = Arc::new(Mutex::new(FeedState::new()));
        let (feed, dispatch) = lock(&state).open(FeedSpec::Search("clip".to_owned()));
        let context = dispatch.expect("search dispatch").context;
        let revisions = lock(&state).subscribe(feed).expect("open feed revision");
        let (sender, outcomes) = mpsc::unbounded_channel();
        Self {
            state,
            feed,
            context,
            revisions,
            sender,
            outcomes,
        }
    }

    pub(super) fn sinks(&self) -> OutcomeSinks {
        OutcomeSinks {
            state: Arc::clone(&self.state),
            bootstrap: test_bootstrap().0,
            candidates: None,
        }
    }

    pub(super) fn queue_progress(&self, count: u64) {
        let keys = Keys::generate();
        for index in 1..=count {
            self.queue_event(video_note(&keys, &format!("clip-{index}"), index));
        }
    }

    pub(super) fn queue_event(&self, event: Event) {
        self.sender
            .send(RetrievalOutcome::Progress {
                context: self.context.clone(),
                event: Box::new(event),
            })
            .expect("live progress receiver");
    }

    pub(super) fn queue_completion(&self, events: Vec<Event>) {
        self.sender
            .send(RetrievalOutcome::Completed {
                context: self.context.clone(),
                result: Ok(events),
                cursor: None,
                complete: true,
                purpose: RetrievalPurpose::Head,
            })
            .expect("live completion receiver");
    }
}
