//! Session identity held beside the account-scoped event pool.

use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::{Event, EventId, PublicKey};
use std::collections::HashSet;

/// Whose session the pool holds.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewerScope {
    /// A query that neither claims nor changes the viewer.
    #[default]
    Unknown,
    /// A signed-out main feed.
    SignedOut,
    /// A main feed opened by this viewer.
    SignedIn(PublicKey),
}

pub(crate) struct EventCacheSession {
    generation: SessionGeneration,
    viewer: ViewerScope,
    admitted: Option<HashSet<EventId>>,
}

impl EventCacheSession {
    pub(crate) fn initial() -> Self {
        Self {
            generation: SessionGeneration::initial(),
            viewer: ViewerScope::Unknown,
            admitted: None,
        }
    }

    pub(crate) fn generation(&self) -> SessionGeneration {
        self.generation
    }

    pub(crate) fn matches(&self, generation: SessionGeneration) -> bool {
        self.generation == generation
    }

    pub(crate) fn reset(&mut self, generation: SessionGeneration) {
        self.generation = generation;
        self.viewer = ViewerScope::Unknown;
        self.admitted = Some(HashSet::new());
    }

    pub(crate) fn adopt(&mut self, viewer: ViewerScope) -> bool {
        if viewer == ViewerScope::Unknown {
            return false;
        }
        let replaced = self.viewer != ViewerScope::Unknown && self.viewer != viewer;
        self.viewer = viewer;
        if replaced {
            self.admitted = Some(HashSet::new());
        }
        replaced
    }

    pub(crate) fn admit(&mut self, events: &[Event]) {
        if let Some(admitted) = &mut self.admitted {
            admitted.extend(events.iter().map(|event| event.id));
        }
    }

    pub(crate) fn retain_admitted(&self, events: &mut Vec<Event>) {
        if let Some(admitted) = &self.admitted {
            events.retain(|event| admitted.contains(&event.id));
        }
    }
}
