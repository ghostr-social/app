use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo, RelayReadResult};
use nostr_sdk::Event;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub(crate) const TARGET_RELAY: &str = "wss://target.example";

pub(crate) struct RepostTargetIo {
    wrapper: Event,
    original: Event,
    deletion: Option<Event>,
    pub(crate) used_hint: AtomicBool,
    pub(crate) used_deletion_hint: AtomicBool,
    target_failure: AtomicBool,
    target_empty: AtomicBool,
}

impl RepostTargetIo {
    pub(crate) fn new(wrapper: Event, original: Event) -> Arc<Self> {
        Arc::new(Self::value(wrapper, original))
    }

    fn value(wrapper: Event, original: Event) -> Self {
        Self {
            wrapper,
            original,
            deletion: None,
            used_hint: AtomicBool::new(false),
            used_deletion_hint: AtomicBool::new(false),
            target_failure: AtomicBool::new(false),
            target_empty: AtomicBool::new(false),
        }
    }

    pub(crate) fn failing(wrapper: Event, original: Event) -> Arc<Self> {
        let io = Self::value(wrapper, original);
        io.target_failure.store(true, Ordering::Relaxed);
        Arc::new(io)
    }

    pub(crate) fn empty_once(wrapper: Event, original: Event) -> Arc<Self> {
        let io = Self::value(wrapper, original);
        io.target_empty.store(true, Ordering::Relaxed);
        Arc::new(io)
    }

    pub(crate) fn with_deletion(wrapper: Event, original: Event, deletion: Event) -> Arc<Self> {
        let mut io = Self::value(wrapper, original);
        io.deletion = Some(deletion);
        Arc::new(io)
    }
}

impl RelayIo for RepostTargetIo {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, RelayReadResult> {
        Box::pin(async move { self.events_for(&request).map(RelayReadResult::complete) })
    }

    fn broadcast(&self, _: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

impl RepostTargetIo {
    fn events_for(&self, request: &RelayReadIo) -> anyhow::Result<Vec<Event>> {
        if request.filter.match_event(&self.wrapper) {
            return Ok(vec![self.wrapper.clone()]);
        }
        let hinted = request.relays.iter().any(|relay| relay == TARGET_RELAY);
        if hinted && request.filter.match_event(&self.original) {
            return self.target_events();
        }
        Ok(self.deletion_events(request, hinted))
    }

    fn target_events(&self) -> anyhow::Result<Vec<Event>> {
        if self.target_failure.swap(false, Ordering::Relaxed) {
            return Err(anyhow::anyhow!("target relay offline"));
        }
        if self.target_empty.swap(false, Ordering::Relaxed) {
            return Ok(Vec::new());
        }
        self.used_hint.store(true, Ordering::Relaxed);
        Ok(vec![self.original.clone()])
    }

    fn deletion_events(&self, request: &RelayReadIo, hinted: bool) -> Vec<Event> {
        let Some(deletion) = &self.deletion else {
            return Vec::new();
        };
        if !hinted || !request.filter.match_event(deletion) {
            return Vec::new();
        }
        self.used_deletion_hint.store(true, Ordering::Relaxed);
        vec![deletion.clone()]
    }
}
