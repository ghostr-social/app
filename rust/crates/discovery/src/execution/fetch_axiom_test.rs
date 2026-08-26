use super::*;

impl FetchedEvents {
    pub(crate) fn fresh(events: Vec<Event>) -> Self {
        let fresh_boundary = wire_page_boundary(&events);
        Self {
            events,
            fresh_boundary,
            wire_complete: true,
        }
    }
}
