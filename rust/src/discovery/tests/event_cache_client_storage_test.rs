//! Root cause of the parity gap. nostr-sdk's `Client::default()` builds
//! `MemoryDatabase::default()`, whose `events` option is *false*: it
//! remembers ids and stores no event, so every Rust query was a cold
//! network round. ndk answers from cache UNION network for the whole
//! session (MemCacheManager with cacheRead/cacheWrite in
//! lib/platform/nostr/build_ndk.dart), which is a one-directional
//! membership advantage no relay coverage can close. The engine's
//! client must store what it receives.

use crate::discovery::event_cache::client_with_event_cache;
use crate::discovery::tests::event_cache_support::{ids, note, notes};
use nostr_sdk::prelude::*;

async fn stored(client: &Client) -> Vec<Event> {
    client
        .database()
        .query(vec![notes()])
        .await
        .expect("the database answers a query")
        .to_vec()
}

#[tokio::test]
async fn the_engine_client_keeps_the_events_it_receives() {
    let client = client_with_event_cache();

    client
        .database()
        .save_event(&note(300))
        .await
        .expect("the database accepts an event");

    assert_eq!(ids(&stored(&client).await), ids(&[note(300)]));
}

#[tokio::test]
async fn the_sdk_default_client_keeps_nothing() {
    let client = Client::default();

    client
        .database()
        .save_event(&note(300))
        .await
        .expect("the default database rejects without erroring");

    assert!(
        stored(&client).await.is_empty(),
        "Client::default() stores ids, not events"
    );
}
