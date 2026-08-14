use crate::plan_executor::RepostRetryDelta;
use crate::scheduler::deferred_reposts::DeferredRepostBook;
use crate::tests::scheduler_support::context;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};

#[test]
fn deferred_reposts_evict_the_oldest_event_at_the_global_count_bound() {
    let feed = context("following");
    let mut book = DeferredRepostBook::default();
    let events: Vec<_> = (1..=129).map(wrapper_at).collect();
    for event in &events {
        book.apply(&feed, deferred(event.clone()));
    }

    let batch = book.batch(&feed);

    assert_eq!(book.retained_len(), 128);
    assert_eq!(batch[0].id, events[1].id);
}

fn deferred(event: Event) -> RepostRetryDelta {
    RepostRetryDelta {
        considered: Vec::new(),
        deferred: vec![event],
    }
}

fn wrapper_at(created_at: u64) -> Event {
    EventBuilder::new(Kind::Custom(16), "")
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("wrapper")
}
