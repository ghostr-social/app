//! The membership fix. Every query files what it fetched, so a repeat
//! of the same query answers with the rows the session already holds
//! plus whatever the relays added.

use crate::discovery::tests::event_cache_support::{cache, note, notes, timestamps};

#[tokio::test]
async fn a_repeated_query_answers_with_the_stored_rows_and_the_new_one() {
    let cache = cache();
    cache.union(&notes(), vec![note(100), note(200)]).await;

    let answered = cache.union(&notes(), vec![note(300)]).await;

    assert_eq!(
        timestamps(&answered),
        vec![300, 200, 100],
        "the relays' answer first, then the pool newest-first"
    );
}

#[tokio::test]
async fn a_row_the_relays_repeat_is_answered_once() {
    let cache = cache();
    cache.union(&notes(), vec![note(100)]).await;

    let answered = cache.union(&notes(), vec![note(100), note(200)]).await;

    assert_eq!(timestamps(&answered), vec![100, 200]);
}
