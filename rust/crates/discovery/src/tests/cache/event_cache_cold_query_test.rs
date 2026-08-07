//! A cold query must behave exactly as it does today. An empty pool
//! adds no row and reorders none, so the first query of a session is
//! still the relays' answer alone, in arrival order — the order the
//! feed's own assembly then sorts (feed_assembly.rs).

use crate::tests::event_cache_support::{cache, note, notes, timestamps};

#[tokio::test]
async fn an_empty_pool_answers_with_the_network_rows_alone() {
    let cache = cache();
    let fetched = vec![note(300), note(100), note(200)];

    let answered = cache.union(&notes(), fetched.clone()).await;

    assert_eq!(answered, fetched, "nothing added, nothing reordered");
}

#[tokio::test]
async fn an_empty_pool_answering_an_empty_page_stays_empty() {
    let cache = cache();

    let answered = cache.union(&notes(), Vec::new()).await;

    assert!(timestamps(&answered).is_empty());
}
