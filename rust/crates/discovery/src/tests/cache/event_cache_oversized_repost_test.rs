use crate::content::parsing::MAX_REPOSTABLE_EVENT_BYTES;
use crate::session_generation::SessionGeneration;
use crate::tests::event_cache_support::{bounded_cache, ids, note, notes};
use nostr_sdk::{Event, EventBuilder, Keys, Kind};

#[tokio::test]
async fn an_oversized_repost_flows_fresh_without_entering_the_cache() {
    let cache = bounded_cache(2);
    cache.union(&notes(), vec![note(100), note(200)]).await;
    let wrapper = oversized_wrapper();
    let reposts = nostr_sdk::Filter::new().kind(Kind::Custom(16));

    let fresh = cache.union(&reposts, vec![wrapper.clone()]).await;

    assert_eq!(ids(&fresh), vec![wrapper.id]);
    assert!(cache
        .stored_for(SessionGeneration::initial(), &reposts)
        .await
        .expect("current session")
        .is_empty());
    assert_eq!(cache.union(&notes(), Vec::new()).await.len(), 2);
}

fn oversized_wrapper() -> Event {
    EventBuilder::new(Kind::Custom(16), "x".repeat(MAX_REPOSTABLE_EVENT_BYTES + 1))
        .sign_with_keys(&Keys::generate())
        .expect("wrapper")
}
