use crate::plan_executor::RepostRetryDelta;
use crate::scheduler::deferred_reposts::DeferredRepostBook;
use crate::tests::scheduler_support::context;
use nostr_sdk::{EventBuilder, Keys, Kind};

#[test]
fn repeatedly_unsettled_repost_eventually_leaves_the_retry_book() {
    let feed = context("following");
    let mut book = DeferredRepostBook::default();
    let wrapper = EventBuilder::new(Kind::Custom(16), "")
        .sign_with_keys(&Keys::generate())
        .expect("wrapper");
    book.apply(&feed, delta(Vec::new(), vec![wrapper]));

    for _ in 0..16 {
        let attempted = book.batch(&feed);
        if attempted.is_empty() {
            break;
        }
        let considered = attempted.iter().map(|event| event.id).collect();
        book.apply(&feed, delta(considered, attempted));
    }

    assert!(book.batch(&feed).is_empty());
}

fn delta(considered: Vec<nostr_sdk::EventId>, deferred: Vec<nostr_sdk::Event>) -> RepostRetryDelta {
    RepostRetryDelta {
        considered,
        deferred,
    }
}
