//! What `Client::fetch_events_from` keeps when several relays answer one
//! filter. nostr-sdk collects into `Events::new(&filters)`, a set capped
//! at the single filter's `limit` (`OverCapacityPolicy::Last`), so the
//! limit bounds the *union* across relays and the oldest events fall out.
//! The engine instead drains the relay stream so the filter limit applies
//! independently to each relay before the results are merged.

use nostr_sdk::prelude::*;

fn note(keys: &Keys, created_at: u64) -> Event {
    EventBuilder::text_note(format!("note {created_at}"))
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(keys)
        .expect("fixture event")
}

#[test]
fn the_filter_limit_caps_the_union_and_drops_the_oldest() {
    let keys = Keys::generate();
    let filter = Filter::new().kind(Kind::TextNote).limit(2);
    let mut collected = Events::new(&[filter]);

    // Two relays answering the same filter, each within its own limit.
    collected.insert(note(&keys, 300));
    collected.insert(note(&keys, 200));
    collected.insert(note(&keys, 100));

    assert_eq!(collected.len(), 2);
    let kept: Vec<u64> = collected
        .iter()
        .map(|event| event.created_at.as_u64())
        .collect();
    assert_eq!(kept, vec![300, 200]);
}

#[test]
fn an_unlimited_filter_keeps_every_relays_answer() {
    let keys = Keys::generate();
    let mut collected = Events::new(&[Filter::new().kind(Kind::TextNote)]);

    collected.insert(note(&keys, 300));
    collected.insert(note(&keys, 200));
    collected.insert(note(&keys, 100));

    assert_eq!(collected.len(), 3);
}
