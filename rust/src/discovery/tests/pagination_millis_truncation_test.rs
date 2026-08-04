//! Dart hands the pagination cutoff as a DateTime; the relay `until` comes
//! from truncating division of its unix milliseconds
//! (lib/platform/nostr/video_discovery_queries.dart `_build`:
//! `olderThan.toUtc().millisecondsSinceEpoch ~/ 1000`).

use nostr_sdk::Timestamp;

use crate::discovery::pagination::older_than_from_unix_millis;

#[test]
fn milliseconds_truncate_down_to_whole_seconds() {
    assert_eq!(older_than_from_unix_millis(1_999), Timestamp::from(1));
    assert_eq!(older_than_from_unix_millis(2_000), Timestamp::from(2));
    assert_eq!(older_than_from_unix_millis(2_001), Timestamp::from(2));
}

#[test]
fn whole_seconds_pass_through_unchanged() {
    assert_eq!(
        older_than_from_unix_millis(1_722_000_000_000),
        Timestamp::from(1_722_000_000)
    );
}

#[test]
fn the_epoch_maps_to_zero() {
    assert_eq!(older_than_from_unix_millis(0), Timestamp::from(0));
}
