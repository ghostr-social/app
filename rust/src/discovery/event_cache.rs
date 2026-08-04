//! The session event pool: the shared Nostr client's own database, read
//! back into every query's answer.
//!
//! Parity source: lib/platform/nostr/build_ndk.dart runs ndk with
//! `MemCacheManager` and `cacheRead`/`cacheWrite` on, so every ndk answer
//! is cache UNION network accumulated over the whole session — including
//! every event the viewer already scrolled past. Rust had no such pool:
//! nostr-sdk's `Client::default()` attaches `MemoryDatabase::default()`,
//! whose `events` option is *false*, so `save_event` only marks an id as
//! seen and `query` answers nothing. Every Rust query was a cold network
//! round, which is the one-directional membership advantage the three
//! device passes measured (third pass: 56 of 60 divergence records list
//! rows ndk served and Rust did not, ordering agreeing every time).
//!
//! Storing events is necessary but not sufficient: verified against the
//! vendored crate, `Client::stream_events*` and `fetch_events*` reach
//! only relays (nostr-relay-pool 0.38 `pool/inner.rs`), never the
//! database. The union is therefore assembled here, from the same filter
//! the query put on the wire. The write side needs no help — the relay
//! layer saves every accepted event on receipt
//! (`relay/inner.rs`, `save_event` after the `NotExistent` check) — but
//! [`EventCache::remember`] files each answer anyway so the pool holds
//! what the feed was actually served, whatever path delivered it.
//!
//! In-memory, not LMDB, and bounded: see [`MAX_CACHED_EVENTS`].

use log::warn;
use nostr_sdk::prelude::{
    Client, ClientBuilder, Event, EventId, Filter, MemoryDatabase, MemoryDatabaseOptions,
    NostrDatabase, PublicKey,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

/// How many events one session keeps.
///
/// The pool is deliberately RAM-only and session-scoped, which is
/// exactly what it is being built to match: ndk's `MemCacheManager` dies
/// with the process, so a persistent store would make the Rust feed a
/// *superset* of the pipeline it is measured against, in a direction no
/// shadow comparison can validate. Persistence also buys little here —
/// the third device pass recorded that the progressive video store does
/// not survive a restart, so rows recovered from a cold database would
/// point at bytes that are gone — and costs a lot: `nostr-lmdb` drags in
/// a C LMDB build that has to cross-compile for every Android ABI, and
/// its map file would land in the same cache directory whose free space
/// the video store already rations (`DEFAULT_RESERVE_BYTES`, 256 MB).
///
/// ndk's cache is unbounded; a phone session cannot be — the same pass
/// recorded an out-of-memory kill. 10k events is roughly twenty full
/// query plans' worth (the wire limits are 80 + 200 + 200 + 200), an
/// order of magnitude more than the 500 rows a feed retains, and a few
/// tens of MB at Nostr event sizes. Past it `nostr-database` evicts the
/// oldest and prunes every index with them.
pub const MAX_CACHED_EVENTS: usize = 10_000;

/// The shared client the engine runs on: one whose database actually
/// stores the events it receives.
pub fn client_with_event_cache() -> Client {
    ClientBuilder::new()
        .database(session_event_database(MAX_CACHED_EVENTS))
        .build()
}

/// A bounded in-memory event store. `MemoryDatabaseOptions::default()`
/// stores no events at all, so both fields are named on purpose.
pub fn session_event_database(max_events: usize) -> MemoryDatabase {
    MemoryDatabase::with_opts(MemoryDatabaseOptions {
        events: true,
        max_events: Some(max_events),
    })
}

/// Whose session the pool holds.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewerScope {
    /// The request names no viewer — a search, hashtag or profile feed,
    /// or a relay-list chase. It neither claims nor changes the scope.
    #[default]
    Unknown,
    /// A signed-out main feed.
    SignedOut,
    /// A main feed opened by this viewer.
    SignedIn(PublicKey),
}

/// Read side of the client's database, scoped to one viewer.
pub struct EventCache {
    database: Arc<dyn NostrDatabase>,
    viewer: Mutex<ViewerScope>,
}

impl EventCache {
    /// Reads the client's own database — the store the relay layer
    /// writes every received event into, so browsing fills the pool.
    pub fn of(client: &Client) -> Self {
        Self::new(client.database().clone())
    }

    pub fn new(database: Arc<dyn NostrDatabase>) -> Self {
        Self {
            database,
            viewer: Mutex::new(ViewerScope::Unknown),
        }
    }

    /// One query's answer: everything the relays streamed, in arrival
    /// order, plus the rows this session already holds for the same
    /// filter and the relays did not repeat. An empty pool changes
    /// nothing, so a cold query behaves exactly as it did before.
    pub async fn union(&self, filter: &Filter, fetched: Vec<Event>) -> Vec<Event> {
        let stored = self.stored(filter).await;
        self.remember(&fetched).await;
        merged(fetched, stored)
    }

    /// Everything stored for one filter, newest first, capped by that
    /// filter's own `limit` exactly as a relay caps its own answer.
    pub async fn stored(&self, filter: &Filter) -> Vec<Event> {
        match self.database.query(vec![filter.clone()]).await {
            Ok(events) => events.to_vec(),
            Err(error) => {
                warn!("The session event pool could not be read: {error}");
                Vec::new()
            }
        }
    }

    /// Files an answer in the pool. Rejections (duplicate, replaced,
    /// ephemeral) are ordinary results, not errors.
    pub async fn remember(&self, events: &[Event]) {
        for event in events {
            if let Err(error) = self.database.save_event(event).await {
                warn!("The session event pool could not store an event: {error}");
            }
        }
    }

    /// Scopes the pool to one viewer and reports whether it emptied it.
    /// The engine outlives a sign-out — the gateway and its client are
    /// installed once per process — so a session that changes identity
    /// must not answer from the previous viewer's rows.
    pub async fn adopt(&self, viewer: ViewerScope) -> bool {
        if !self.replaces_viewer(viewer) {
            return false;
        }
        if let Err(error) = self.database.wipe().await {
            warn!("The session event pool could not be cleared: {error}");
        }
        true
    }

    /// A pool that never had a viewer cannot hold another viewer's rows,
    /// so the first claim of a session keeps what booting gathered.
    fn replaces_viewer(&self, viewer: ViewerScope) -> bool {
        if viewer == ViewerScope::Unknown {
            return false;
        }
        let mut current = locked(&self.viewer);
        let replaced = *current != ViewerScope::Unknown && *current != viewer;
        *current = viewer;
        replaced
    }
}

fn merged(mut fetched: Vec<Event>, stored: Vec<Event>) -> Vec<Event> {
    let fresh: HashSet<EventId> = fetched.iter().map(|event| event.id).collect();
    fetched.extend(stored.into_iter().filter(|event| !fresh.contains(&event.id)));
    fetched
}

fn locked(viewer: &Mutex<ViewerScope>) -> MutexGuard<'_, ViewerScope> {
    viewer
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
