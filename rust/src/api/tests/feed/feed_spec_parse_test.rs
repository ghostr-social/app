//! `parse_feed_spec`: the typed FFI feed spec becomes a `FeedSpec`,
//! with hard errors for unusable values.

use crate::api::feed::mapping::parse_feed_spec;
use crate::api::feed_types::{FfiFeedKind, FfiFeedSpec};
use crate::discovery::feed::spec::FeedSpec;
use nostr_sdk::Keys;

fn spec(kind: FfiFeedKind, value: Option<&str>, viewer: Option<String>) -> FfiFeedSpec {
    FfiFeedSpec {
        kind,
        value: value.map(str::to_owned),
        creators: Vec::new(),
        viewer_pubkey: viewer,
    }
}

fn creator_spec(creators: &[String]) -> FfiFeedSpec {
    FfiFeedSpec {
        creators: creators.to_vec(),
        ..spec(FfiFeedKind::Profile, None, None)
    }
}

#[test]
fn main_feed_takes_the_viewer_pubkey() {
    let viewer = Keys::generate().public_key();
    let parsed = parse_feed_spec(&spec(FfiFeedKind::Main, None, Some(viewer.to_hex())));
    let viewer = Some(viewer);
    assert_eq!(parsed.expect("main parses"), FeedSpec::MainFeed { viewer });
}

/// Signed out Dart names no viewer and the feed stays the unscoped
/// global page the signed-out feed contract serves.
#[test]
fn a_main_feed_without_a_viewer_is_the_signed_out_feed() {
    let parsed = parse_feed_spec(&spec(FfiFeedKind::Main, None, None));
    assert_eq!(
        parsed.expect("main parses"),
        FeedSpec::MainFeed { viewer: None }
    );
}

/// One creator for a profile grid, the whole follow set for the
/// Following feed — both arrive in `creators`.
#[test]
fn profile_feed_names_every_creator() {
    let creators: Vec<_> = (0..2).map(|_| Keys::generate().public_key()).collect();
    let hexes: Vec<String> = creators.iter().map(|key| key.to_hex()).collect();

    let parsed = parse_feed_spec(&creator_spec(&hexes));

    assert_eq!(parsed.expect("profile parses"), FeedSpec::Profile(creators));
}

#[test]
fn hashtag_and_search_feeds_carry_the_value_as_typed() {
    let hashtag = parse_feed_spec(&spec(FfiFeedKind::Hashtag, Some("#Sunset"), None));
    assert_eq!(
        hashtag.expect("hashtag parses"),
        FeedSpec::Hashtag("#Sunset".to_owned())
    );
    let search = parse_feed_spec(&spec(FfiFeedKind::Search, Some("  "), None));
    assert_eq!(
        search.expect("search parses"),
        FeedSpec::Search("  ".to_owned())
    );
}

#[test]
fn unusable_specs_are_rejected() {
    assert!(parse_feed_spec(&spec(FfiFeedKind::Main, None, Some("not-a-key".to_owned()))).is_err());
    assert!(parse_feed_spec(&creator_spec(&[])).is_err());
    assert!(parse_feed_spec(&creator_spec(&["bad".to_owned()])).is_err());
    assert!(parse_feed_spec(&spec(FfiFeedKind::Hashtag, None, None)).is_err());
    assert!(parse_feed_spec(&spec(FfiFeedKind::Search, None, None)).is_err());
}
