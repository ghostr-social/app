//! Search feeds normalize the viewer's query the way Dart does: trimmed
//! and lowercased (`VideoSearchPolicy.normalize`), `#`-queries become
//! hashtag requests, and blank queries never reach a relay — mirrors
//! lib/features/video_catalog/domain/video_search_policy.dart and the
//! query split in discovery_video_search_repository.dart `searchVideos`.

mod feed_support;

use feed_support::empty_graph;
use rust_lib_ghostr::discovery::feed_spec::FeedSpec;

fn search(query: &str) -> FeedSpec {
    FeedSpec::Search(query.to_owned())
}

#[test]
fn feed_spec_search_trims_and_lowercases_the_query() {
    let request = search("  Sunset Surf ").page_request(None, &empty_graph()).expect("request");

    assert_eq!(request.search_query, Some("sunset surf".to_owned()));
    assert!(request.hashtags.is_empty());
}

#[test]
fn feed_spec_search_with_leading_hash_becomes_a_hashtag_request() {
    let request = search("#Cats").page_request(None, &empty_graph()).expect("request");

    assert_eq!(request.hashtags, vec!["cats".to_owned()]);
    assert_eq!(request.search_query, None);
}

#[test]
fn feed_spec_blank_search_never_queries() {
    assert_eq!(search("   ").page_request(None, &empty_graph()), None);
}

#[test]
fn feed_spec_lone_hash_stays_a_text_search() {
    // `normalizeHashtag('#')` yields nothing, so Dart falls through to a
    // plain text search for "#" (video_search_policy.dart `hashtag`).
    let request = search("#").page_request(None, &empty_graph()).expect("request");

    assert_eq!(request.search_query, Some("#".to_owned()));
    assert!(request.hashtags.is_empty());
}
