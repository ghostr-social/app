//! A plain feed request issues the four Dart query shapes in order:
//! dedicated video kinds, a kind-1 note window, the mp4 note hunt, and a
//! NIP-94 file query (lib/platform/nostr/video_discovery_queries.dart).

use serde_json::json;

use crate::discovery::tests::support::filter_json;
use crate::discovery::video_filters::{discovery_filters, DiscoveryRequest};

#[test]
fn plain_feed_builds_four_filters_in_dart_order() {
    assert_eq!(discovery_filters(&DiscoveryRequest::default()).len(), 4);
}

#[test]
fn video_kind_filter_uses_the_narrow_limit_without_scope() {
    let json = filter_json(&discovery_filters(&DiscoveryRequest::default())[0]);

    assert_eq!(json["kinds"], json!([21, 22, 34235, 34236]));
    assert_eq!(json["limit"], json!(80));
    assert!(json.get("search").is_none());
    assert!(json.get("authors").is_none());
    assert!(json.get("until").is_none());
    assert!(json.get("#t").is_none());
}

#[test]
fn note_filter_asks_for_kind_one_at_the_wide_limit() {
    let json = filter_json(&discovery_filters(&DiscoveryRequest::default())[1]);

    assert_eq!(json["kinds"], json!([1]));
    assert_eq!(json["limit"], json!(200));
    assert!(json.get("search").is_none());
}

#[test]
fn note_hunt_filter_searches_for_a_literal_video_mention() {
    let json = filter_json(&discovery_filters(&DiscoveryRequest::default())[2]);

    assert_eq!(json["kinds"], json!([1]));
    assert_eq!(json["limit"], json!(200));
    assert_eq!(json["search"], json!("mp4"));
}

#[test]
fn file_filter_asks_kind_1063_scoped_to_video_mimes() {
    let json = filter_json(&discovery_filters(&DiscoveryRequest::default())[3]);

    assert_eq!(json["kinds"], json!([1063]));
    assert_eq!(json["limit"], json!(200));
    // nostr_sdk stores tag values as an ordered set; same members as Dart.
    assert_eq!(
        json["#m"],
        json!([
            "application/vnd.apple.mpegurl",
            "application/x-mpegurl",
            "video/mp4",
            "video/mpeg",
            "video/quicktime",
            "video/webm",
        ])
    );
}
