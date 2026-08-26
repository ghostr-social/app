use crate::manager::timeline::axiom_test_support::TimelineIncomplete;
use crate::manager::timeline::axiom_test_support::TimelineInput;
use crate::manager::timeline::axiom_test_support::TimelineParse;
use crate::manager::timeline::axiom_test_support::TimelineParser;
use crate::manager::timeline::{TimelineRejection, TimelineTerminal};
use ghostr_engine::media_timeline::MediaTimeline;
use ghostr_engine::media_timeline::TimelineParseControl;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use tokio::sync::mpsc;

pub(crate) struct GatedTimelineParser {
    calls: AtomicUsize,
    started: mpsc::UnboundedSender<usize>,
    released: Arc<(Mutex<Vec<bool>>, Condvar)>,
    first_ready: Option<MediaTimeline>,
    reject_refresh: bool,
}

impl GatedTimelineParser {
    pub(super) fn new(
        first_ready: Option<MediaTimeline>,
        gates: usize,
    ) -> (Arc<Self>, mpsc::UnboundedReceiver<usize>) {
        let (started, receiver) = mpsc::unbounded_channel();
        let parser = Self {
            calls: AtomicUsize::new(0),
            started,
            released: Arc::new((Mutex::new(vec![false; gates]), Condvar::new())),
            first_ready,
            reject_refresh: false,
        };
        (Arc::new(parser), receiver)
    }

    pub(super) fn rejecting_refresh(
        first_ready: MediaTimeline,
    ) -> (Arc<Self>, mpsc::UnboundedReceiver<usize>) {
        let (parser, receiver) = Self::new(Some(first_ready), 2);
        let parser = Arc::try_unwrap(parser).ok().expect("unshared parser");
        (
            Arc::new(Self {
                reject_refresh: true,
                ..parser
            }),
            receiver,
        )
    }

    pub(super) fn release(&self, call: usize) {
        let (released, changed) = &*self.released;
        released.lock().expect("valid test fixture")[call] = true;
        changed.notify_all();
    }
}

impl TimelineParser for GatedTimelineParser {
    fn parse(&self, _input: TimelineInput, _control: &dyn TimelineParseControl) -> TimelineParse {
        let call = self.calls.fetch_add(1, Ordering::AcqRel);
        self.started.send(call).expect("valid test fixture");
        let (released, changed) = &*self.released;
        let mut state = released.lock().expect("valid test fixture");
        while !state[call] {
            state = changed.wait(state).expect("valid test fixture");
        }
        TimelineParse::Completed(match (call, self.first_ready.as_ref()) {
            (0, Some(timeline)) => TimelineTerminal::Ready(Box::new(timeline.clone())),
            _ if self.reject_refresh => TimelineTerminal::Rejected(TimelineRejection::Malformed),
            _ => TimelineTerminal::Incomplete(TimelineIncomplete::Unavailable),
        })
    }
}
