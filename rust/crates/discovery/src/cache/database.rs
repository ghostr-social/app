//! Construction of the bounded, in-memory Nostr event pool.

use nostr_sdk::prelude::{Client, ClientBuilder, MemoryDatabase, MemoryDatabaseOptions};

/// How many events one account session keeps.
pub(super) const MAX_CACHED_EVENTS: usize = 10_000;

/// The shared client keeps only bounded seen IDs. Relay-pool 0.38 still
/// streams `Saved` events, while disabling events keeps late old fetches
/// from recreating deletion or replaceable indexes after reset.
pub fn client_with_event_cache() -> Client {
    ClientBuilder::new()
        .database(MemoryDatabase::with_opts(MemoryDatabaseOptions {
            events: false,
            max_events: Some(MAX_CACHED_EVENTS),
        }))
        .build()
}

/// A bounded in-memory event store. Defaults would store no events.
pub fn session_event_database(max_events: usize) -> MemoryDatabase {
    MemoryDatabase::with_opts(MemoryDatabaseOptions {
        events: true,
        max_events: Some(max_events),
    })
}
