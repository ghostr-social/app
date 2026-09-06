//! Bounded knowledge of successful transform recovery in this store session.
//! Forgetting a key only repeats recovery; it never admits unverified bytes.

use std::collections::{HashSet, VecDeque};
use std::sync::{Mutex, MutexGuard};

const RECOVERED_KEY_LIMIT: usize = 1_024;

#[derive(Default)]
pub(super) struct TransformRecovery {
    known: Mutex<KnownRecovery>,
}

#[derive(Default)]
struct KnownRecovery {
    keys: HashSet<String>,
    order: VecDeque<String>,
}

impl TransformRecovery {
    pub(super) fn contains(&self, key: &str) -> bool {
        self.lock().keys.contains(key)
    }

    pub(super) fn remember(&self, key: &str) {
        let mut known = self.lock();
        if !known.keys.insert(key.to_owned()) {
            return;
        }
        known.order.push_back(key.to_owned());
        if known.order.len() > RECOVERED_KEY_LIMIT {
            if let Some(oldest) = known.order.pop_front() {
                known.keys.remove(&oldest);
            }
        }
    }

    pub(super) fn forget(&self, key: &str) {
        let mut known = self.lock();
        if known.keys.remove(key) {
            known.order.retain(|known| known != key);
        }
    }

    pub(super) fn clear(&self) {
        *self.lock() = KnownRecovery::default();
    }

    fn lock(&self) -> MutexGuard<'_, KnownRecovery> {
        self.known.lock().unwrap_or_else(|error| error.into_inner())
    }
}
