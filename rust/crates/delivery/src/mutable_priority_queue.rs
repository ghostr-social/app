//! Replaceable queue of planned download work.
//!
//! Every delivery event can replace the ordering in place. Work that
//! left the plan disappears, while newly focused work moves to the front.

use crate::manager::admission::origin_key;
use crate::manager::plan::{PlannedTransfer, PlannedTransferId};
use ghostr_engine::tiers::Tier;
use std::collections::{HashSet, VecDeque};

#[derive(Default)]
pub(crate) struct MutablePriorityQueue {
    pending: VecDeque<PlannedTransfer>,
    wanted: HashSet<PlannedTransferId>,
    frontier: Option<Tier>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ForegroundSlots {
    active: usize,
    goal: usize,
}

impl ForegroundSlots {
    pub(crate) fn new(active: usize, goal: usize) -> Self {
        Self { active, goal }
    }

    fn needs_fill(self) -> bool {
        self.active < self.goal
    }
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
        self.frontier = self.pending.front().map(|transfer| transfer.request.tier);
        self.wanted = seen;
    }

    pub fn pop_for_hosts(
        &mut self,
        active_hosts: &HashSet<String>,
        foreground: ForegroundSlots,
    ) -> Option<PlannedTransfer> {
        let frontier = self.frontier?;
        let index = if is_foreground(frontier) {
            self.playback_index(active_hosts, foreground)
        } else {
            self.idle_frontier_index(active_hosts)
                .or_else(|| self.frontier_index())
        }?;
        self.pending.remove(index)
    }

    pub fn pop_for_idle_host(&mut self, active_hosts: &HashSet<String>) -> Option<PlannedTransfer> {
        let index = self.idle_frontier_index(active_hosts)?;
        self.pending.remove(index)
    }

    fn idle_frontier_index(&self, active_hosts: &HashSet<String>) -> Option<usize> {
        let frontier = self.frontier?;
        self.pending.iter().position(|transfer| {
            transfer.request.tier == frontier && !active_hosts.contains(&origin_key(&transfer.url))
        })
    }

    fn frontier_index(&self) -> Option<usize> {
        let frontier = self.frontier?;
        self.pending
            .iter()
            .position(|transfer| transfer.request.tier == frontier)
    }

    fn playback_index(
        &self,
        active_hosts: &HashSet<String>,
        foreground: ForegroundSlots,
    ) -> Option<usize> {
        let idle = || self.idle_foreground_index(active_hosts);
        let foreground_index = || self.foreground_index();
        if foreground.needs_fill() {
            return idle()
                .or_else(foreground_index)
                .or_else(|| self.protected_index());
        }
        self.protected_index()
            .or_else(idle)
            .or_else(foreground_index)
    }

    fn idle_foreground_index(&self, active_hosts: &HashSet<String>) -> Option<usize> {
        self.pending.iter().position(|transfer| {
            is_foreground(transfer.request.tier)
                && !active_hosts.contains(&origin_key(&transfer.url))
        })
    }

    fn foreground_index(&self) -> Option<usize> {
        self.pending
            .iter()
            .position(|transfer| is_foreground(transfer.request.tier))
    }

    fn protected_index(&self) -> Option<usize> {
        self.pending
            .iter()
            .position(|transfer| transfer.request.tier == Tier::T2Startability)
    }

    #[cfg(test)]
    pub fn wanted(&self) -> HashSet<PlannedTransferId> {
        self.wanted.clone()
    }

    pub fn wanted_len(&self) -> usize {
        self.wanted.len()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
        self.wanted.clear();
        self.frontier = None;
    }
}

fn is_foreground(tier: Tier) -> bool {
    matches!(tier, Tier::T0PlaybackEmergency | Tier::T1CurrentTail)
}
