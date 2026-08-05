//! Scheduler pagination from relay events that can become feed rows.

use crate::discovery::event_parsing::video_post_from_event;
use crate::discovery::pagination::next_page_cursor;
use nostr_sdk::{Event, Timestamp};

pub(crate) fn playable_cursor(events: &[Event]) -> Option<Timestamp> {
    next_page_cursor(
        events
            .iter()
            .filter(|event| video_post_from_event(event).is_some())
            .map(|event| event.created_at),
    )
}
