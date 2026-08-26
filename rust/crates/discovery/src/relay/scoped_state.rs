//! Shared result state and cancellation cleanup for scoped relay reads.

use crate::retrieval_types::EventProgress;
use core::sync::atomic::{AtomicBool, Ordering};
use nostr_sdk::{ClientMessage, Event, EventId, Relay, SubscriptionId};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(super) struct EventSink {
    events: Arc<Mutex<CollectedEvents>>,
    progress: Option<EventProgress>,
}

#[derive(Default)]
struct CollectedEvents {
    ids: HashSet<EventId>,
    events: Vec<Event>,
    limit: usize,
    overflowed: bool,
}

impl EventSink {
    pub(super) fn new(progress: Option<EventProgress>, limit: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(CollectedEvents {
                limit,
                ..CollectedEvents::default()
            })),
            progress,
        }
    }

    pub(super) async fn record(&self, event: Event) {
        if !self.insert(event.clone()) {
            return;
        }
        if let Some(progress) = &self.progress {
            let _ = progress.send(event).await;
        }
    }

    pub(super) fn snapshot(&self) -> Vec<Event> {
        self.lock().events.clone()
    }

    pub(super) fn overflowed(&self) -> bool {
        self.lock().overflowed
    }

    fn insert(&self, event: Event) -> bool {
        let mut collected = self.lock();
        if collected.ids.contains(&event.id) {
            return false;
        }
        if collected.events.len() >= collected.limit {
            collected.overflowed = true;
            return false;
        }
        collected.ids.insert(event.id);
        collected.events.push(event);
        true
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, CollectedEvents> {
        self.events
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

#[derive(Clone)]
pub(super) struct CloseGuard {
    relay: Relay,
    id: SubscriptionId,
    closed: Arc<AtomicBool>,
}

impl CloseGuard {
    pub(super) fn new(relay: Relay, id: SubscriptionId) -> Self {
        Self {
            relay,
            id,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Drop for CloseGuard {
    fn drop(&mut self) {
        if !self.closed.swap(true, Ordering::AcqRel) {
            let _ = self.relay.send_msg(ClientMessage::close(self.id.clone()));
        }
    }
}
