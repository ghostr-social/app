//! Each production feed shape builds its canonical discovery request:
//! the main feed queries unscoped, profiles scope to the creator, and
//! hashtag feeds carry the normalized tag.

mod feed_support;

use feed_support::empty_graph;
use ghostr_discovery::cache::ViewerScope;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::query::video_filters::{DiscoveryFlow, DiscoveryRequest};
use nostr_sdk::{Keys, Timestamp};

/// Nothing narrows the wire filter — no authors, tag or term — and the
/// only thing the main feed stamps is the session pool's viewer scope,
/// which never reaches the wire.
#[test]
fn feed_spec_main_feed_requests_everything_unscoped() {
    let viewer = Keys::generate().public_key();

    let request = FeedSpec::MainFeed {
        viewer: Some(viewer),
    }
    .page_request(None, &empty_graph());

    assert_eq!(
        request,
        Some(DiscoveryRequest {
            viewer: ViewerScope::SignedIn(viewer),
            flow: DiscoveryFlow::Continuous,
            ..DiscoveryRequest::default()
        })
    );
}

#[test]
fn feed_spec_passes_the_pagination_cursor_through() {
    let viewer = Some(Keys::generate().public_key());
    let cursor = Timestamp::from(1_700_000_000);

    let request = FeedSpec::MainFeed { viewer }.page_request(Some(cursor), &empty_graph());

    assert_eq!(request.expect("request").older_than, Some(cursor));
}

#[test]
fn feed_spec_profile_feed_scopes_to_the_creator() {
    let creator = Keys::generate().public_key();

    let request = FeedSpec::Profile(vec![creator]).page_request(None, &empty_graph());

    assert_eq!(request.expect("request").authors, vec![creator]);
}

#[test]
fn feed_spec_hashtag_feed_carries_the_normalized_tag() {
    let request = FeedSpec::Hashtag("#Cats ".to_owned()).page_request(None, &empty_graph());

    let request = request.expect("request");
    assert_eq!(request.hashtags, vec!["cats".to_owned()]);
    assert_eq!(request.search_query, None);
}

#[test]
fn feed_spec_empty_hashtag_never_queries() {
    assert_eq!(
        FeedSpec::Hashtag("#".to_owned()).page_request(None, &empty_graph()),
        None
    );
}
