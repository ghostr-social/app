use nostr_sdk::Event;

/// Events collected from selected relays and whether every relay proved EOSE.
#[derive(Debug)]
pub struct RelayReadResult {
    pub events: Vec<Event>,
    pub complete: bool,
}

impl RelayReadResult {
    pub fn complete(events: Vec<Event>) -> Self {
        Self {
            events,
            complete: true,
        }
    }

    pub(crate) fn incomplete(events: Vec<Event>) -> Self {
        Self {
            events,
            complete: false,
        }
    }
}
