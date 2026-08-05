//! The public FFI filter shape preserves every Nostr query field on the wire.

use nostr_sdk::Filter;
use rust_lib_ghostr::api::event_types::{FfiNostrEventFilter, FfiNostrTagFilter};
use serde_json::json;

const AUTHOR: &str = "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const EVENT: &str = "1111111111111111111111111111111111111111111111111111111111111111";

#[test]
fn ffi_filter_preserves_supported_nostr_fields() {
    let input = FfiNostrEventFilter {
        kinds: vec![7, 1111],
        authors: vec![AUTHOR.to_owned()],
        event_tags: vec![EVENT.to_owned()],
        tag_filters: vec![
            FfiNostrTagFilter {
                name: "A".to_owned(),
                values: vec!["root".to_owned()],
            },
            FfiNostrTagFilter {
                name: "t".to_owned(),
                values: vec!["Rust".to_owned()],
            },
        ],
        limit: 25,
        until: Some(1_700_000_000),
        search: Some("nostr video".to_owned()),
    };

    let filter = Filter::try_from(input).expect("valid filter");
    let wire = serde_json::to_value(filter).expect("filter serializes");

    assert_eq!(wire["kinds"], json!([7, 1111]));
    assert_eq!(wire["authors"], json!([AUTHOR]));
    assert_eq!(wire["#e"], json!([EVENT]));
    assert_eq!(wire["#A"], json!(["root"]));
    assert_eq!(wire["#t"], json!(["Rust"]));
    assert_eq!(wire["limit"], json!(25));
    assert_eq!(wire["until"], json!(1_700_000_000));
    assert_eq!(wire["search"], json!("nostr video"));
}
