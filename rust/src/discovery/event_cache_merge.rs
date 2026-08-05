//! Stable union of fresh relay events and admitted cached rows.

use nostr_sdk::{Event, EventId};
use std::collections::HashSet;

pub(crate) fn merged(mut fetched: Vec<Event>, stored: Vec<Event>) -> Vec<Event> {
    let fresh: HashSet<EventId> = fetched.iter().map(|event| event.id).collect();
    fetched.extend(
        stored
            .into_iter()
            .filter(|event| !fresh.contains(&event.id)),
    );
    fetched
}
