//! The pool answers a query only with rows that query's own filter
//! matches, and only as many as its `limit` allows — it behaves like one
//! more relay. Nothing therefore reaches feed assembly
//! (event_parsing.rs, feed_spec.rs) that a network answer could not have
//! carried; stored rows travel the identical parsing and filtering path.

use crate::discovery::tests::event_cache_support::{cache, note, notes, timestamps};
use nostr_sdk::prelude::*;

#[tokio::test]
async fn a_stored_row_never_answers_a_filter_it_does_not_match() {
    let cache = cache();
    cache.union(&notes(), vec![note(100)]).await;

    let files = Filter::new().kind(Kind::Custom(1063));
    let answered = cache.union(&files, Vec::new()).await;

    assert!(
        answered.is_empty(),
        "a kind-1 note cannot answer a file-metadata query"
    );
}

#[tokio::test]
async fn the_querys_own_limit_caps_what_the_pool_adds() {
    let cache = cache();
    cache
        .union(&notes(), vec![note(100), note(200), note(300)])
        .await;

    let answered = cache.union(&notes().limit(2), Vec::new()).await;

    assert_eq!(
        timestamps(&answered),
        vec![300, 200],
        "the newest rows, capped like a relay's own answer"
    );
}
