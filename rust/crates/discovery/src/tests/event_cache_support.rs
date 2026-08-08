//! Shared fixtures for the session event pool's tests.

use crate::cache::{session_event_database, EventCache};
use nostr_sdk::prelude::*;
use std::sync::Arc;

/// One deterministic author for every fixture, so two runs of the same
/// note produce the same event id and the pool can deduplicate them.
fn keys() -> Keys {
    Keys::parse("1111111111111111111111111111111111111111111111111111111111111111")
        .expect("valid fixture secret key")
}

/// A kind-1 note whose id is a function of its timestamp alone.
pub(crate) fn note(created_at: u64) -> Event {
    EventBuilder::text_note(format!("note {created_at}"))
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&keys())
        .expect("fixture event")
}

/// Every fixture note is a kind-1, so this filter matches them all.
pub(crate) fn notes() -> Filter {
    Filter::new().kind(Kind::TextNote)
}

pub(crate) fn timestamps(events: &[Event]) -> Vec<u64> {
    events
        .iter()
        .map(|event| event.created_at.as_u64())
        .collect()
}

/// Identity, not the whole event: a schnorr signature carries fresh
/// auxiliary randomness, so two signings of one note differ by `sig`
/// while sharing the id every deduplication keys on.
pub(crate) fn ids(events: &[Event]) -> Vec<EventId> {
    events.iter().map(|event| event.id).collect()
}

pub(crate) fn cache() -> EventCache {
    bounded_cache(64)
}

pub(crate) fn bounded_cache(max_events: usize) -> EventCache {
    EventCache::new(Arc::new(session_event_database(max_events)))
}
