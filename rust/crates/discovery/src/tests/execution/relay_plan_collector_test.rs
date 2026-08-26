//! The executor keeps every relay's answer. `Client::fetch_events_from`
//! collects into `Events::new(&filters)`, a set capped at the single
//! filter's `limit` (see `relay_fetch_union_limit_test`), so the limit
//! bounds the *union across relays* and the oldest events fall out.
//! Draining the pool's stream preserves each relay's contribution before
//! the engine merges the union.

use crate::relay::io::axiom_test_support::drain_events;

use nostr_sdk::prelude::*;

fn note(keys: &Keys, created_at: u64) -> Event {
    EventBuilder::text_note(format!("note {created_at}"))
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(keys)
        .expect("fixture event")
}

/// Three events answering a limit-2 filter: one relay's newest two plus
/// an older one only the second relay had.
fn answers(keys: &Keys) -> Vec<Event> {
    vec![note(keys, 300), note(keys, 200), note(keys, 100)]
}

#[tokio::test]
async fn every_relays_answer_survives_the_filter_limit() {
    let keys = Keys::generate();
    let streamed = answers(&keys);

    let collected = drain_events(tokio_stream::iter(streamed.clone())).await;

    assert_eq!(collected.len(), 3);
    let kept: Vec<u64> = collected.iter().map(|e| e.created_at.as_u64()).collect();
    assert_eq!(kept, vec![300, 200, 100]);
}

#[tokio::test]
async fn the_capped_collection_would_have_dropped_the_oldest() {
    let keys = Keys::generate();
    let mut capped = Events::new(&[Filter::new().kind(Kind::TextNote).limit(2)]);
    for event in answers(&keys) {
        capped.insert(event);
    }

    assert_eq!(capped.len(), 2, "the union, not the per-relay answer");
    assert!(!capped.iter().any(|event| event.created_at.as_u64() == 100));
}
