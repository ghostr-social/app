//! The executor keeps every relay's answer. `Client::fetch_events_from`
//! collects into `Events::new(&filters)`, a set capped at the single
//! filter's `limit` (see relay_fetch_union_limit_test), so the limit
//! bounds the *union across relays* and the oldest events fall out.
//! Draining the pool's stream preserves each relay's contribution before
//! the engine merges the union.

use crate::plan_executor::PlanFailure;
use crate::relay_io::drain_events;
use crate::relay_plan_collector::collect_events;
use crate::search_queries::QueryRole;
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

#[tokio::test]
async fn additive_failure_keeps_the_page_unsettled() {
    let primary_event = note(&Keys::generate(), 100);
    let primary = tokio::spawn({
        let event = primary_event.clone();
        async move { Ok(vec![event]) }
    });
    let additive = tokio::spawn(async { Err(PlanFailure::new("additive failed")) });

    let failure = collect_events(vec![
        (QueryRole::Primary, primary),
        (QueryRole::Additive, additive),
    ])
    .await
    .expect_err("a lossy page cannot commit its cursor");

    assert_eq!(failure.message, "additive failed");
}
