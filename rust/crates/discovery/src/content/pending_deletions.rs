//! Best-effort retention for deletions whose targets are not retained yet.

use super::deletion_index::DeletionKey;
use std::collections::{BTreeMap, HashMap};

const PENDING_DELETION_RETENTION: usize = 4_000;

#[derive(Debug)]
struct PendingClaim {
    deleted_at: u64,
    admitted_at: u64,
}

#[derive(Debug)]
pub(super) struct PendingDeletions {
    claims: HashMap<DeletionKey, PendingClaim>,
    order: BTreeMap<u64, DeletionKey>,
    next_admission: u64,
    retention: usize,
}

impl Default for PendingDeletions {
    fn default() -> Self {
        Self {
            claims: HashMap::new(),
            order: BTreeMap::new(),
            next_admission: 0,
            retention: PENDING_DELETION_RETENTION,
        }
    }
}

impl PendingDeletions {
    pub(super) fn insert(&mut self, key: DeletionKey, deleted_at: u64) -> bool {
        if let Some(current) = self.claims.get_mut(&key) {
            return replace_if_newer(&mut current.deleted_at, deleted_at);
        }
        self.admit(key, deleted_at);
        self.trim();
        true
    }

    pub(super) fn take(&mut self, key: &DeletionKey) -> Option<u64> {
        let claim = self.claims.remove(key)?;
        self.order.remove(&claim.admitted_at);
        Some(claim.deleted_at)
    }

    pub(super) fn get(&self, key: &DeletionKey) -> Option<u64> {
        self.claims.get(key).map(|claim| claim.deleted_at)
    }

    fn admit(&mut self, key: DeletionKey, deleted_at: u64) {
        let admitted_at = self.next_admission;
        self.next_admission += 1;
        self.order.insert(admitted_at, key.clone());
        self.claims.insert(
            key,
            PendingClaim {
                deleted_at,
                admitted_at,
            },
        );
    }

    fn trim(&mut self) {
        while self.claims.len() > self.retention {
            let (_, key) = self
                .order
                .pop_first()
                .expect("pending claims share one admission order");
            self.claims.remove(&key);
        }
    }
}

fn replace_if_newer(current: &mut u64, incoming: u64) -> bool {
    if incoming <= *current {
        return false;
    }
    *current = incoming;
    true
}

#[cfg(test)]
#[path = "pending_deletions_axiom_test.rs"]
mod axiom_test_support;
