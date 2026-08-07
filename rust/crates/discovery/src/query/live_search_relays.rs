//! Clone-shared search relay configuration.

use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub(crate) struct LiveSearchRelays {
    relays: Arc<RwLock<Vec<String>>>,
}

impl LiveSearchRelays {
    pub(crate) fn new(relays: Vec<String>) -> Self {
        Self {
            relays: Arc::new(RwLock::new(relays)),
        }
    }

    pub(crate) fn replace(&self, relays: Vec<String>) {
        *self
            .relays
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = relays;
    }

    pub(crate) fn snapshot(&self) -> Vec<String> {
        self.relays
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}
