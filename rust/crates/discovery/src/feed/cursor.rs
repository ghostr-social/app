//! Pagination cursors for canonical feeds and raw retrieval pages.

use crate::content::parsing::video_post_from_event;
use crate::feed::pagination::{next_page_cursor, NEXT_PAGE_BACKSTEP_SECS};
use crate::query::video_filters::{FILE_EVENT_KIND, VIDEO_EVENT_KINDS, VIDEO_NOTE_KIND};
use nostr_sdk::{Event, Timestamp};

/// Cursor for background feed prefetch, based only on playable rows.
pub(crate) fn playable_cursor(events: &[Event]) -> Option<Timestamp> {
    next_page_cursor(
        events
            .iter()
            .filter(|event| video_post_from_event(event).is_some())
            .map(|event| event.created_at),
    )
}

/// Cursor from every content event the relay returned, even when the event
/// cannot become a playable row. Profile enrichment must not move it.
pub fn retrieval_cursor(events: &[Event]) -> Option<Timestamp> {
    let mut oldest: [Option<Timestamp>; 3] = [None; 3];
    for event in events {
        let Some(family) = content_family(event) else {
            continue;
        };
        oldest[family] =
            Some(oldest[family].map_or(event.created_at, |current| current.min(event.created_at)));
    }
    oldest.into_iter().flatten().max().map(backstep)
}

/// Conservative cursor across independently capped wire-filter answers.
pub(crate) fn wire_retrieval_cursor<'a>(
    pages: impl IntoIterator<Item = &'a [Event]>,
) -> Option<Timestamp> {
    pages
        .into_iter()
        .filter_map(oldest_content)
        .max()
        .map(backstep)
}

fn oldest_content(events: &[Event]) -> Option<Timestamp> {
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
    match kind {
        VIDEO_NOTE_KIND => Some(1),
        FILE_EVENT_KIND => Some(2),
        _ => None,
    }
}

fn backstep(oldest: Timestamp) -> Timestamp {
    Timestamp::from(oldest.as_u64().saturating_sub(NEXT_PAGE_BACKSTEP_SECS))
}
