//! Pagination cursors for canonical feeds and raw retrieval pages.

use crate::content::reposts::feed_post_from_event;
use crate::feed::pagination::{next_page_cursor, NEXT_PAGE_BACKSTEP_SECS};
use crate::query::video_filters::{
    DELETION_KIND, FILE_EVENT_KIND, GENERIC_REPOST_KIND, REPOST_KIND, VIDEO_EVENT_KINDS,
    VIDEO_NOTE_KIND,
};
use nostr_sdk::{Event, Timestamp};

const SINGLE_KIND_FAMILIES: [u16; 5] = [
    VIDEO_NOTE_KIND,
    FILE_EVENT_KIND,
    REPOST_KIND,
    GENERIC_REPOST_KIND,
    DELETION_KIND,
];

/// Cursor for background feed prefetch, based only on playable rows.
pub(crate) fn playable_cursor(events: &[Event]) -> Option<Timestamp> {
    next_page_cursor(
        events
            .iter()
            .filter(|event| feed_post_from_event(event).is_some())
            .map(|event| event.created_at),
    )
}

/// Cursor from every content event the relay returned, even when the event
/// cannot become a playable row. Profile enrichment must not move it.
pub fn retrieval_cursor(events: &[Event]) -> Option<Timestamp> {
    let mut oldest: [Option<Timestamp>; 6] = [None; 6];
    for event in events {
        let Some(family) = content_family(event) else {
            continue;
        };
        oldest[family] =
            Some(oldest[family].map_or(event.created_at, |current| current.min(event.created_at)));
    }
    oldest.into_iter().flatten().max().and_then(backstep)
}

/// Conservative cursor across independently capped wire-filter answers.
pub(crate) fn wire_retrieval_cursor(
    boundaries: impl IntoIterator<Item = Timestamp>,
) -> Option<Timestamp> {
    boundaries.into_iter().max().and_then(backstep)
}

pub(crate) fn wire_page_boundary(events: &[Event]) -> Option<Timestamp> {
    events
        .iter()
        .filter(|event| content_family(event).is_some())
        .map(|event| event.created_at)
        .min()
}

fn content_family(event: &Event) -> Option<usize> {
    let kind = event.kind.as_u16();
    if VIDEO_EVENT_KINDS.contains(&kind) {
        return Some(0);
    }
    SINGLE_KIND_FAMILIES
        .iter()
        .position(|candidate| *candidate == kind)
        .map(|index| index + 1)
}

fn backstep(oldest: Timestamp) -> Option<Timestamp> {
    oldest
        .as_u64()
        .checked_sub(NEXT_PAGE_BACKSTEP_SECS)
        .map(Timestamp::from)
}
