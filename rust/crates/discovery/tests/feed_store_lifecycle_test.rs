use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use nostr_sdk::Keys;

#[tokio::test]
async fn feed_lifecycle_notifies_and_never_reuses_a_stale_id() {
    let viewer = Keys::generate().public_key();
    let mut store = FeedStore::new();
    let first = store.open_feed(FeedSpec::MainFeed {
        viewer: Some(viewer),
    });
    assert!(matches!(store.spec(first), FeedSpec::MainFeed { .. }));
    let mut first_updates = store.subscribe(first).expect("subscription");
    first_updates.borrow_and_update();
    store.touch(first);
    assert!(first_updates.has_changed().expect("open feed"));
    first_updates.borrow_and_update();

    store.close_feed(first);
    assert!(first_updates.changed().await.is_err());
    let second = store.open_feed(FeedSpec::MainFeed { viewer: None });
    let mut second_updates = store.subscribe(second).expect("subscription");
    store.reset_session();
    assert!(second_updates.changed().await.is_err());

    let third = store.open_feed(FeedSpec::MainFeed { viewer: None });
    assert!(third.0 > second.0);
}
