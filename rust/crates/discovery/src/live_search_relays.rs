//! Clone-shared search relay configuration.

use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub(super) struct LiveSearchRelays {
    relays: Arc<RwLock<Vec<String>>>,
}

impl LiveSearchRelays {
    pub(super) fn new(relays: Vec<String>) -> Self {
        Self {
            relays: Arc::new(RwLock::new(relays)),
        }
    }

    pub(super) fn replace(&self, relays: Vec<String>) {
        *self
            .relays
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = relays;
    }

    pub(super) fn snapshot(&self) -> Vec<String> {
        self.relays
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}
