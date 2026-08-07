//! The session event pool is bounded. Past the bound the oldest rows
//! fall out, and falling out must stay a plain eviction:
//! a tombstoned id would make the client drop that event on arrival
//! (nostr-relay-pool 0.38 relay/inner.rs skips a `Deleted` status), which
//! is the very defect this pool exists to fix.

use crate::cache::session_event_database;
use crate::tests::event_cache_support::{bounded_cache, note, notes, timestamps};
use nostr_sdk::prelude::*;

#[tokio::test]
async fn the_pool_never_grows_past_its_bound() {
    let cache = bounded_cache(3);
    let fetched: Vec<_> = (1..=6).map(|step| note(step * 100)).collect();

    cache.union(&notes(), fetched).await;
    let stored = cache.stored(&notes()).await;

    assert_eq!(
        timestamps(&stored),
        vec![600, 500, 400],
        "the newest rows survive; the oldest fall out"
    );
}

#[tokio::test]
async fn an_evicted_row_is_not_tombstoned() {
    let database = session_event_database(2);
    for created_at in [100, 200, 300] {
        database
            .save_event(&note(created_at))
            .await
            .expect("stored");
    }

    let status = database.check_id(&note(100).id).await.expect("checked");

    assert_eq!(status, DatabaseEventStatus::NotExistent);
}
