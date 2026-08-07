//! Replaceable queue of planned download work.
//!
//! Every delivery event can replace the ordering in place. Work that
//! left the plan disappears, while newly focused work moves to the front.

use ghostr_engine::ChunkId;
use crate::manager::plan::PlannedTransfer;
use std::collections::{HashSet, VecDeque};

#[derive(Default)]
pub(crate) struct MutablePriorityQueue {
    pending: VecDeque<PlannedTransfer>,
    wanted: HashSet<ChunkId>,
}

impl MutablePriorityQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace(&mut self, transfers: Vec<PlannedTransfer>) {
        let mut seen = HashSet::new();
        self.pending = transfers
            .into_iter()
            .filter(|transfer| seen.insert(transfer.request.chunk.clone()))
            .collect();
        self.wanted = seen;
    }

    pub fn pop(&mut self) -> Option<PlannedTransfer> {
        self.pending.pop_front()
    }

    pub fn wanted(&self) -> HashSet<ChunkId> {
        self.wanted.clone()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
        self.wanted.clear();
    }
}
