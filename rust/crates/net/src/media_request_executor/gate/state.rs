use super::{MediaRequestGate, RequestLease};
use crate::media_request_executor::MediaRequestLimits;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use std::collections::HashMap;
use tokio::sync::oneshot;

mod active;
use active::ActiveCapacity;

pub(super) struct GateState {
    pub(super) limits: MediaRequestLimits,
    active: ActiveCapacity,
    authorities: HashMap<RequestAuthority, ActiveCapacity>,
    waiters: Vec<Waiter>,
    next_sequence: u64,
}

struct Waiter {
    sequence: u64,
    authority: RequestAuthority,
    priority: PreemptionAuthority,
    granted: oneshot::Sender<RequestLease>,
}

impl GateState {
    pub(super) fn new(limits: MediaRequestLimits) -> Self {
        Self {
            limits,
            active: ActiveCapacity::default(),
            authorities: HashMap::new(),
            waiters: Vec::new(),
            next_sequence: 0,
        }
    }

    pub(super) fn enqueue(
        &mut self,
        authority: RequestAuthority,
        priority: PreemptionAuthority,
        granted: oneshot::Sender<RequestLease>,
    ) -> u64 {
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .expect("media request sequence exhausted");
        self.waiters.push(Waiter {
            sequence: self.next_sequence,
            authority,
            priority,
            granted,
        });
        self.next_sequence
    }

    pub(super) fn cancel(&mut self, sequence: u64) {
        self.waiters.retain(|waiter| waiter.sequence != sequence);
    }

    pub(super) fn take_grants(
        &mut self,
        gate: &MediaRequestGate,
    ) -> Vec<(oneshot::Sender<RequestLease>, RequestLease)> {
        let mut grants = Vec::new();
        while let Some(index) = self.next_admissible() {
            let waiter = self.waiters.remove(index);
            self.claim(&waiter.authority, waiter.priority);
            let lease = RequestLease::new(gate.clone(), waiter.authority, waiter.priority);
            grants.push((waiter.granted, lease));
        }
        grants
    }

    pub(super) fn release(&mut self, authority: &RequestAuthority, priority: PreemptionAuthority) {
        self.authorities
            .get_mut(authority)
            .expect("released media authority is not active")
            .release(priority);
        self.active.release(priority);
        self.authorities.retain(|_, active| active.total() > 0);
    }

    pub(super) fn active_for(&self, authority: &RequestAuthority) -> usize {
        self.authority_capacity(authority).total()
    }

    fn next_admissible(&self) -> Option<usize> {
        self.waiters
            .iter()
            .enumerate()
            .filter(|(_, waiter)| self.available(waiter))
            .min_by_key(|(_, waiter)| (priority_rank(waiter.priority), waiter.sequence))
            .map(|(index, _)| index)
    }

    fn available(&self, waiter: &Waiter) -> bool {
        self.active.available(self.limits.global(), waiter.priority)
            && self
                .authority_capacity(&waiter.authority)
                .available(self.limits.per_authority(), waiter.priority)
    }

    fn authority_capacity(&self, authority: &RequestAuthority) -> ActiveCapacity {
        self.authorities.get(authority).copied().unwrap_or_default()
    }

    fn claim(&mut self, authority: &RequestAuthority, priority: PreemptionAuthority) {
        self.active.claim(priority);
        self.authorities
            .entry(authority.clone())
            .or_default()
            .claim(priority);
    }
}

fn priority_rank(priority: PreemptionAuthority) -> u8 {
    match priority {
        PreemptionAuthority::PlaybackCritical => 0,
        PreemptionAuthority::Transition => 1,
        PreemptionAuthority::Speculative => 2,
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
