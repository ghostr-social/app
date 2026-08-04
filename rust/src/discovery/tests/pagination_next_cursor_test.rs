//! The next-page cursor steps one second below the oldest fetched post and
//! disappears when a page fetched nothing — the cursor advances by what was
//! fetched, not what survived filtering
//! (lib/features/video_catalog/domain/filtered_video_feed_repository.dart
//! `_nextCursor`: `oldest.subtract(const Duration(seconds: 1))`).

use nostr_sdk::Timestamp;

use crate::discovery::pagination::next_page_cursor;

fn t(secs: u64) -> Timestamp {
    Timestamp::from(secs)
}

#[test]
fn cursor_is_one_second_before_the_oldest_post() {
    assert_eq!(next_page_cursor([t(100)]), Some(t(99)));
}

#[test]
fn oldest_wins_regardless_of_fetch_order() {
    assert_eq!(next_page_cursor([t(300), t(120), t(250)]), Some(t(119)));
}

#[test]
fn empty_page_exhausts_the_feed() {
    assert_eq!(next_page_cursor([]), None);
}

#[test]
fn cursor_saturates_at_the_epoch() {
    // Dart would step to -1s (pre-1970) here; a relay `until` cannot go
    // below zero, so the Rust cursor floors at the epoch instead.
    assert_eq!(next_page_cursor([t(0)]), Some(t(0)));
}
