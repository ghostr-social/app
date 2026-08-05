//! Relays match tag values exactly, so hashtag queries expand to every
//! case form publishers commonly write: as-typed, lower, UPPER, Title,
//! deduplicated in that order and across the whole requested set.

use crate::discovery::hashtags::{hashtag_filter_values, hashtag_query_variants};

#[test]
fn typed_case_leads_and_duplicates_collapse() {
    assert_eq!(hashtag_query_variants("Surf"), ["Surf", "surf", "SURF"]);
    assert_eq!(hashtag_query_variants("skate"), ["skate", "SKATE", "Skate"]);
    assert_eq!(hashtag_query_variants("BTC"), ["BTC", "btc", "Btc"]);
}

#[test]
fn leading_hash_and_whitespace_are_stripped() {
    assert_eq!(hashtag_query_variants("#foo"), ["foo", "FOO", "Foo"]);
    assert_eq!(hashtag_query_variants("  #Foo  "), ["Foo", "foo", "FOO"]);
}

#[test]
fn unicode_tags_case_fold_like_dart() {
    assert_eq!(
        hashtag_query_variants("çedilha"),
        ["çedilha", "ÇEDILHA", "Çedilha"]
    );
}

#[test]
fn empty_input_yields_no_variants() {
    assert!(hashtag_query_variants("").is_empty());
    assert!(hashtag_query_variants("#").is_empty());
    assert!(hashtag_query_variants("   ").is_empty());
}

#[test]
fn filter_values_dedupe_across_the_whole_hashtag_set() {
    let values = hashtag_filter_values(&["surf".into(), "SURF".into()]);

    assert_eq!(values, ["surf", "SURF", "Surf"]);
}
