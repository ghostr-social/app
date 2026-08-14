//! Globally bounded raw reposts awaiting target or deletion settlement.

use crate::content::parsing::MAX_REPOSTABLE_EVENT_BYTES;
use crate::plan_executor::RepostRetryDelta;
use crate::retrieval_types::FeedContext;
use nostr_sdk::{Event, EventId, JsonUtil};
use std::collections::HashMap;

const MAX_DEFERRED_REPOSTS: usize = 128;
const MAX_DEFERRED_REPOST_BYTES: usize = 4 * 1024 * 1024;
const MAX_DEFERRED_EVENT_BYTES: usize = MAX_REPOSTABLE_EVENT_BYTES * 2;
const MAX_DEFERRED_ATTEMPTS: u8 = 3;
pub(crate) const MAX_REPOSTS_PER_ATTEMPT: usize = 32;

type EntryKey = (FeedContext, EventId);

#[derive(Debug)]
struct DeferredEntry {
    event: Event,
    wire_bytes: usize,
    order: u64,
    attempts: u8,
}

#[derive(Debug, Default)]
pub(crate) struct DeferredRepostBook {
    entries: HashMap<EntryKey, DeferredEntry>,
    bytes: usize,
    next_order: u64,
}

impl DeferredRepostBook {
    pub(crate) fn batch(&self, context: &FeedContext) -> Vec<Event> {
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|((stored, _), _)| stored == context)
            .map(|(_, entry)| entry)
            .collect();
        entries.sort_by_key(|entry| entry.order);
        entries
            .into_iter()
            .take(MAX_REPOSTS_PER_ATTEMPT)
            .map(|entry| entry.event.clone())
            .collect()
    }

    pub(crate) fn apply(&mut self, context: &FeedContext, delta: RepostRetryDelta) -> bool {
        let mut attempts = self.take_attempts(context, delta.considered);
        for event in delta.deferred {
            let key = (context.clone(), event.id);
            let retained = self.remove(&key).map(|entry| entry.attempts);
            let attempts = attempts.remove(&event.id).or(retained).unwrap_or(0);
            if attempts < MAX_DEFERRED_ATTEMPTS {
                self.insert(context.clone(), event, attempts);
            }
        }
        self.has_pending(context)
    }

    pub(crate) fn has_pending(&self, context: &FeedContext) -> bool {
        self.entries.keys().any(|(stored, _)| stored == context)
    }

    pub(crate) fn remove_context(&mut self, context: &FeedContext) {
        let keys: Vec<_> = self
            .entries
            .keys()
            .filter(|(stored, _)| stored == context)
            .cloned()
            .collect();
        for key in keys {
            let _ = self.remove(&key);
        }
    }

    pub(crate) fn reset(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }

    #[cfg(test)]
    pub(crate) fn retained_len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub(crate) fn retained_bytes(&self) -> usize {
        self.bytes
    }

    fn take_attempts(
        &mut self,
        context: &FeedContext,
        considered: Vec<EventId>,
    ) -> HashMap<EventId, u8> {
        considered
            .into_iter()
            .map(|id| {
                let key = (context.clone(), id);
                let attempts = self
                    .remove(&key)
                    .map_or(1, |entry| entry.attempts.saturating_add(1));
                (id, attempts)
            })
            .collect()
    }

    fn insert(&mut self, context: FeedContext, event: Event, attempts: u8) {
        let Some(wire_bytes) = retained_wire_bytes(&event) else {
            return;
        };
        let key = (context, event.id);
        let _ = self.remove(&key);
        self.bytes = self.bytes.saturating_add(wire_bytes);
        self.next_order = self.next_order.wrapping_add(1);
        self.entries.insert(
            key,
            DeferredEntry {
                event,
                wire_bytes,
                order: self.next_order,
                attempts,
            },
        );
        self.trim();
    }

    fn remove(&mut self, key: &EntryKey) -> Option<DeferredEntry> {
        let entry = self.entries.remove(key);
        if let Some(entry) = &entry {
            self.bytes = self.bytes.saturating_sub(entry.wire_bytes);
        }
        entry
    }

    fn trim(&mut self) {
        while self.entries.len() > MAX_DEFERRED_REPOSTS || self.bytes > MAX_DEFERRED_REPOST_BYTES {
            let key = self
                .eviction_key()
                .expect("a nonempty deferred book always has an eviction candidate");
            let _ = self.remove(&key);
        }
    }

    fn eviction_key(&self) -> Option<EntryKey> {
        self.entries
            .iter()
            .min_by(|left, right| {
                left.1
                    .event
                    .created_at
                    .cmp(&right.1.event.created_at)
                    .then_with(|| right.0 .1.cmp(&left.0 .1))
            })
            .map(|(key, _)| key.clone())
    }
}

fn retained_wire_bytes(event: &Event) -> Option<usize> {
    let fields = event
        .tags
        .iter()
        .flat_map(|tag| tag.as_slice())
        .fold(event.content.len(), |total, value| {
            total.saturating_add(value.len())
        });
    if fields > MAX_DEFERRED_EVENT_BYTES {
        return None;
    }
    let wire_bytes = event.as_json().len();
    (wire_bytes <= MAX_DEFERRED_EVENT_BYTES).then_some(wire_bytes)
}
