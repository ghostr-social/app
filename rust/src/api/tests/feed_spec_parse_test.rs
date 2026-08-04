//! `parse_feed_spec`: the FFI's stringly feed spec becomes the typed
//! `FeedSpec`, with hard errors for unusable input.

use crate::api::feed_mapping::parse_feed_spec;
use crate::api::feed_types::FfiFeedSpec;
use crate::discovery::feed_spec::FeedSpec;
use nostr_sdk::Keys;

fn spec(kind: &str, value: Option<&str>, viewer: Option<String>) -> FfiFeedSpec {
    FfiFeedSpec {
        kind: kind.to_owned(),
        value: value.map(str::to_owned),
        viewer_pubkey: viewer,
    }
}

#[test]
fn main_feed_takes_the_viewer_pubkey() {
    let viewer = Keys::generate().public_key();
    let parsed = parse_feed_spec(&spec("main", None, Some(viewer.to_hex())));
    let viewer = Some(viewer);
    assert_eq!(parsed.expect("main parses"), FeedSpec::MainFeed { viewer });
}

/// Signed out Dart names no viewer and the feed stays the unscoped
/// global page ndk serves (rust_feed_spec_builder.dart).
#[test]
fn a_main_feed_without_a_viewer_is_the_signed_out_feed() {
    let parsed = parse_feed_spec(&spec("main", None, None));
    assert_eq!(parsed.expect("main parses"), FeedSpec::MainFeed { viewer: None });
}

#[test]
fn profile_feed_names_the_creator_in_value() {
    let creator = Keys::generate().public_key();
    let parsed = parse_feed_spec(&spec("profile", Some(&creator.to_hex()), None));
    assert_eq!(parsed.expect("profile parses"), FeedSpec::Profile(creator));
}

#[test]
fn hashtag_and_search_feeds_carry_the_value_as_typed() {
    let hashtag = parse_feed_spec(&spec("hashtag", Some("#Sunset"), None));
    assert_eq!(hashtag.expect("hashtag parses"), FeedSpec::Hashtag("#Sunset".to_owned()));
    let search = parse_feed_spec(&spec("search", Some("  "), None));
    assert_eq!(search.expect("search parses"), FeedSpec::Search("  ".to_owned()));
}

#[test]
fn unusable_specs_are_rejected() {
    assert!(parse_feed_spec(&spec("trending", None, None)).is_err());
    assert!(parse_feed_spec(&spec("main", None, Some("not-a-key".to_owned()))).is_err());
    assert!(parse_feed_spec(&spec("profile", None, None)).is_err());
    assert!(parse_feed_spec(&spec("profile", Some("bad"), None)).is_err());
    assert!(parse_feed_spec(&spec("hashtag", None, None)).is_err());
    assert!(parse_feed_spec(&spec("search", None, None)).is_err());
}
