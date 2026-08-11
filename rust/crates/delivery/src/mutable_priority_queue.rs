//! Replaceable queue of planned download work.
//!
//! Every delivery event can replace the ordering in place. Work that
//! left the plan disappears, while newly focused work moves to the front.

use crate::manager::admission::origin_key;
use crate::manager::plan::{PlannedTransfer, PlannedTransferId};
use std::collections::{HashSet, VecDeque};

#[derive(Default)]
pub(crate) struct MutablePriorityQueue {
    pending: VecDeque<PlannedTransfer>,
    wanted: HashSet<PlannedTransferId>,
}

impl MutablePriorityQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace(&mut self, transfers: Vec<PlannedTransfer>) {
        let mut seen = HashSet::new();
        self.pending = transfers
            .into_iter()
            .filter(|transfer| seen.insert(transfer.id()))
            .collect();
        self.wanted = seen;
    }

    pub fn pop_for_hosts(&mut self, active_hosts: &HashSet<String>) -> Option<PlannedTransfer> {
        let index = self
            .pending
            .iter()
            .position(|transfer| !active_hosts.contains(&origin_key(&transfer.url)))
            .unwrap_or(0);
        self.pending.remove(index)
    }

    pub fn pop_for_idle_host(&mut self, active_hosts: &HashSet<String>) -> Option<PlannedTransfer> {
        let index = self
            .pending
            .iter()
            .position(|transfer| !active_hosts.contains(&origin_key(&transfer.url)))?;
        self.pending.remove(index)
    }

    pub fn wanted(&self) -> HashSet<PlannedTransferId> {
        self.wanted.clone()
    }

    pub fn wanted_len(&self) -> usize {
        self.wanted.len()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
        self.wanted.clear();
    }
}
