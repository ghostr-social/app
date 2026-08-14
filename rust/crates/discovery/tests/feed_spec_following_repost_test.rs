mod feed_support;

use feed_support::{empty_graph, repost, video_note};
use ghostr_discovery::content::reposts::feed_post_from_event;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use ghostr_discovery::query::video_filters::{DiscoveryFlow, RepostAdmission};
use nostr_sdk::Keys;

#[test]
fn following_requests_and_admits_reposts_by_followed_people() {
    let creator = Keys::generate();
    let followed = Keys::generate();
    let original = video_note(&creator, "clip", 10);
    let wrapper = repost(&followed, &original, 30);
    let spec = FeedSpec::Following {
        viewer: None,
        follows: vec![followed.public_key()],
    };
    let request = spec.page_request(None, &empty_graph()).expect("request");
    assert_eq!(request.reposts, RepostAdmission::Included);
    assert_eq!(request.flow, DiscoveryFlow::Continuous);

    let mut store = FeedStore::new();
    let feed = store.open_feed(spec);
    store.ingest_first_page(
        feed,
        vec![feed_post_from_event(&wrapper).expect("repost parses")],
        &empty_graph(),
    );

    assert_eq!(store.posts(feed).len(), 1);
}
