//! Relay bookkeeping is deliberately separate from account cache rows.
//! The client remembers bounded IDs but cannot rebuild deletion or
//! replaceable indexes after reset; `EventCache` owns queryable events.

use crate::cache::{client_with_event_cache, EventCache};
use crate::tests::event_cache_support::{ids, note, notes};
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
async fn the_engine_client_keeps_only_seen_ids() {
    let client = client_with_event_cache();
    let event = note(300);

    client
        .database()
        .save_event(&event)
        .await
        .expect("the database accepts an event");

    assert!(ids(&stored(&client).await).is_empty());
    assert_eq!(
        client.database().check_id(&event.id).await.unwrap(),
        DatabaseEventStatus::Saved
    );
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

#[tokio::test]
async fn the_session_cache_does_not_write_through_the_client_database() {
    let client = client_with_event_cache();
    let cache = EventCache::of(&client);
    let event = note(400);

    cache.remember(std::slice::from_ref(&event)).await;

    assert_eq!(ids(&cache.stored(&notes()).await), vec![event.id]);
    assert!(stored(&client).await.is_empty());
}
