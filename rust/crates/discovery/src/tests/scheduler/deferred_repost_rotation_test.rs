use crate::plan_executor::RepostRetryDelta;
use crate::scheduler::deferred_reposts::DeferredRepostBook;
use crate::tests::scheduler_support::context;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};

#[test]
fn failed_attempt_rotates_behind_an_unattempted_deferred_repost() {
    let feed = context("following");
    let mut book = DeferredRepostBook::default();
    let events: Vec<_> = (1..=33).map(wrapper_at).collect();
    book.apply(&feed, delta(Vec::new(), events.clone()));
    let attempted = book.batch(&feed);

    book.apply(
        &feed,
        delta(attempted.iter().map(|event| event.id).collect(), attempted),
    );

    assert_eq!(book.batch(&feed)[0].id, events[32].id);
}

fn delta(considered: Vec<nostr_sdk::EventId>, deferred: Vec<Event>) -> RepostRetryDelta {
    RepostRetryDelta {
        considered,
        deferred,
    }
}

fn wrapper_at(created_at: u64) -> Event {
    EventBuilder::new(Kind::Custom(16), "")
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("wrapper")
}
