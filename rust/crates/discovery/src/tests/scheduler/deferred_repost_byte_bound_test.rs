use crate::plan_executor::RepostRetryDelta;
use crate::scheduler::deferred_reposts::DeferredRepostBook;
use crate::tests::scheduler_support::context;
use nostr_sdk::{Event, EventBuilder, EventId, Keys, Kind, Tag};

#[test]
fn oversized_wrapper_cannot_evict_a_legitimate_deferred_repost() {
    let feed = context("following");
    let mut book = DeferredRepostBook::default();
    let legitimate = small_wrapper();
    book.apply(&feed, deferred(legitimate.clone()));
    book.apply(&feed, deferred(large_wrapper('a')));
    book.apply(&feed, deferred(large_wrapper('b')));

    assert!(book.retained_bytes() <= 4 * 1024 * 1024);
    assert_eq!(book.retained_len(), 1);
    assert_eq!(book.batch(&feed)[0].id, legitimate.id);
}

fn small_wrapper() -> Event {
    EventBuilder::new(Kind::Custom(16), "")
        .custom_created_at(nostr_sdk::Timestamp::from(1))
        .sign_with_keys(&Keys::generate())
        .expect("wrapper")
}

fn deferred(event: Event) -> RepostRetryDelta {
    RepostRetryDelta {
        considered: Vec::new(),
        deferred: vec![event],
    }
}

fn large_wrapper(fill: char) -> Event {
    let target = EventId::all_zeros().to_hex();
    let padding: String = std::iter::repeat_n(fill, 2_200_000).collect();
    EventBuilder::new(Kind::Custom(16), "")
        .tags([tag(&["e", &target]), tag(&["x", &padding])])
        .sign_with_keys(&Keys::generate())
        .expect("wrapper")
}

fn tag(values: &[&str]) -> Tag {
    Tag::parse(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("tag")
}
