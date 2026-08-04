//! `parse_feed_id`: feed handles cross the FFI as the numeric strings
//! `ffi_open_feed` returned; anything else is rejected.

use crate::api::feed_mapping::parse_feed_id;
use crate::discovery::feed_store::FeedId;

#[test]
fn numeric_strings_resolve_to_feed_ids() {
    assert_eq!(parse_feed_id("7").expect("parses"), FeedId(7));
}

#[test]
fn non_numeric_handles_are_rejected() {
    assert!(parse_feed_id("").is_err());
    assert!(parse_feed_id("feed-7").is_err());
    assert!(parse_feed_id("-1").is_err());
}
