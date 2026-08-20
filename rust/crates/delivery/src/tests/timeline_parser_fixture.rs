use crate::manager::timeline::{
    TimelineIncomplete, TimelineInput, TimelineParse, TimelineParser, TimelineRejection,
    TimelineTerminal,
};
use ghostr_engine::media_timeline::MediaTimeline;
use ghostr_engine::media_timeline::TimelineParseControl;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    pub(crate) fn new(
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

    pub(crate) fn rejecting_refresh(
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

    pub(crate) fn release(&self, call: usize) {
        let (released, changed) = &*self.released;
        released.lock().unwrap()[call] = true;
        changed.notify_all();
    }
}

impl TimelineParser for GatedTimelineParser {
    fn parse(&self, _input: TimelineInput, _control: &dyn TimelineParseControl) -> TimelineParse {
        let call = self.calls.fetch_add(1, Ordering::AcqRel);
        self.started.send(call).unwrap();
        let (released, changed) = &*self.released;
        let mut state = released.lock().unwrap();
        while !state[call] {
            state = changed.wait(state).unwrap();
        }
        TimelineParse::Completed(match (call, self.first_ready.as_ref()) {
            (0, Some(timeline)) => TimelineTerminal::Ready(timeline.clone()),
            _ if self.reject_refresh => TimelineTerminal::Rejected(TimelineRejection::Malformed),
            _ => TimelineTerminal::Incomplete(TimelineIncomplete::Unavailable),
        })
    }
}
