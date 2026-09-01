//! Shared fixtures for the session event pool's tests.

use crate::cache::{session_event_database, EventCache};
use core::sync::atomic::{AtomicU64, Ordering};
use nostr_sdk::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

/// One deterministic author for every fixture, so two runs of the same
/// note produce the same event id and the pool can deduplicate them.
fn keys() -> Keys {
    Keys::parse("1111111111111111111111111111111111111111111111111111111111111111")
        .expect("valid fixture secret key")
}

/// A kind-1 note whose id is a function of its timestamp alone.
pub(crate) fn note(created_at: u64) -> Event {
    note_with_content(created_at, format!("note {created_at}"))
}

pub(crate) fn note_with_content(created_at: u64, content: String) -> Event {
    EventBuilder::text_note(content)
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

pub(crate) fn persistent_cache(root: &Path, database: Arc<dyn NostrDatabase>) -> EventCache {
    EventCache::persistent_with_database(database, root)
}

pub(crate) struct PersistentCacheFixture(PathBuf);

impl PersistentCacheFixture {
    pub(crate) fn new(label: &str) -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let name = format!("ghostr-{label}-{}-{sequence}", std::process::id());
        Self(std::env::temp_dir().join(name))
    }

    pub(crate) fn cache(&self) -> EventCache {
        EventCache::persistent(&self.0)
    }

    pub(crate) fn root(&self) -> &Path {
        &self.0
    }

    pub(crate) fn snapshot(&self) -> PathBuf {
        crate::cache::persistence::snapshot_path(&self.0)
    }
}

impl Drop for PersistentCacheFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
