//! The discovery constants are explicit product decisions: accepted
//! kinds and mimes, the mp4 hunt term, and narrow/wide limits.

use crate::discovery::video_filters::{
    FEED_VIDEO_LIMIT, FILE_EVENT_KIND, VIDEO_EVENT_KINDS, VIDEO_FILE_MIME_TYPES,
    VIDEO_NOTE_HUNT_TERM, VIDEO_NOTE_KIND, WIDE_QUERY_LIMIT,
};

#[test]
fn video_event_kinds_match_the_dart_list() {
    assert_eq!(VIDEO_EVENT_KINDS, [21, 22, 34235, 34236]);
}

#[test]
fn note_and_file_kinds_match_the_dart_queries() {
    assert_eq!(VIDEO_NOTE_KIND, 1);
    assert_eq!(FILE_EVENT_KIND, 1063);
}

#[test]
fn video_file_mime_types_match_the_dart_list() {
    assert_eq!(
        VIDEO_FILE_MIME_TYPES,
        [
            "video/mp4",
            "video/webm",
            "video/quicktime",
            "video/mpeg",
            "application/x-mpegurl",
            "application/vnd.apple.mpegurl",
        ]
    );
}

#[test]
fn limits_and_hunt_term_match_the_dart_queries() {
    assert_eq!(FEED_VIDEO_LIMIT, 80);
    assert_eq!(WIDE_QUERY_LIMIT, 200);
    assert_eq!(VIDEO_NOTE_HUNT_TERM, "mp4");
}
