use crate::content::social_graph::SocialGraph;
use crate::feed::spec::FeedSpec;
use crate::feed::store::FeedStore;
use crate::tests::feed_store_support::page;
use nostr_sdk::Keys;

#[test]
fn one_busy_coordinate_cannot_evict_every_other_video() {
    let mut crowded = page(3_000, 2_001);
    for post in &mut crowded {
        post.kind = 34_235;
        post.identifier = Some("crowded".to_owned());
        post.published_identifier = Some("crowded".to_owned());
    }
    let quiet = page(999, 1).remove(0);
    let quiet_id = quiet.event_id.clone();
    crowded.push(quiet);
    let graph = SocialGraph::new(Keys::generate().public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Search("video".to_owned()));

    store.ingest_first_page(feed, crowded, &graph);

    assert!(store
        .posts(feed)
        .iter()
        .any(|post| post.event_id == quiet_id));
}
