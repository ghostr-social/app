//! Search feeds trim and lowercase the viewer's query. Hashtag queries
//! become hashtag requests, and blank queries never reach a relay.

use crate::feed::spec::FeedSpec;
use crate::tests::feed_support::empty_graph;

fn search(query: &str) -> FeedSpec {
    FeedSpec::Search(query.to_owned())
}

#[test]
fn feed_spec_search_trims_and_lowercases_the_query() {
    let request = search("  Sunset Surf ")
        .page_request(None, &empty_graph())
        .expect("request");

    assert_eq!(request.search_query, Some("sunset surf".to_owned()));
    assert!(request.hashtags.is_empty());
}

#[test]
fn feed_spec_search_with_leading_hash_becomes_a_hashtag_request() {
    let request = search("#Cats")
        .page_request(None, &empty_graph())
        .expect("request");

    assert_eq!(request.hashtags, vec!["cats".to_owned()]);
    assert_eq!(request.search_query, None);
}

#[test]
fn feed_spec_blank_search_never_queries() {
    assert_eq!(search("   ").page_request(None, &empty_graph()), None);
}

#[test]
fn feed_spec_lone_hash_stays_a_text_search() {
    // A lone hash has no hashtag body, so it remains a plain-text search.
    let request = search("#")
        .page_request(None, &empty_graph())
        .expect("request");

    assert_eq!(request.search_query, Some("#".to_owned()));
    assert!(request.hashtags.is_empty());
}
