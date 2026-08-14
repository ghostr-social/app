use crate::plan_executor::RepostRetryDelta;
use crate::scheduler::deferred_reposts::DeferredRepostBook;
use crate::tests::scheduler_support::context;
use nostr_sdk::{EventBuilder, Keys, Kind};

#[test]
fn context_close_and_session_reset_release_deferred_reposts() {
    let first = context("following-a");
    let second = context("following-b");
    let mut book = DeferredRepostBook::default();
    book.apply(&first, deferred(wrapper()));
    book.apply(&second, deferred(wrapper()));

    book.remove_context(&first);
    assert!(book.batch(&first).is_empty());
    assert_eq!(book.retained_len(), 1);

    book.reset();
    assert!(book.batch(&second).is_empty());
    assert_eq!(book.retained_bytes(), 0);
}

fn deferred(event: nostr_sdk::Event) -> RepostRetryDelta {
    RepostRetryDelta {
        considered: Vec::new(),
        deferred: vec![event],
    }
}

fn wrapper() -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(16), "")
        .sign_with_keys(&Keys::generate())
        .expect("wrapper")
}
