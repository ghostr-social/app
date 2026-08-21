use super::{MediaRequestGate, RequestLease};
use crate::media_request_executor::MediaRequestLimits;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use std::collections::HashMap;
use tokio::sync::oneshot;

pub(super) struct GateState {
    pub(super) limits: MediaRequestLimits,
    active: usize,
    authorities: HashMap<RequestAuthority, usize>,
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
            active: 0,
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
        while self.active < self.limits.global() {
            let Some(index) = self.next_admissible() else {
                break;
            };
            let waiter = self.waiters.remove(index);
            self.claim(&waiter.authority);
            let lease = RequestLease::new(gate.clone(), waiter.authority);
            grants.push((waiter.granted, lease));
        }
        grants
    }

    pub(super) fn release(&mut self, authority: &RequestAuthority) {
        self.active = self.active.saturating_sub(1);
        if let Some(active) = self.authorities.get_mut(authority) {
            *active = active.saturating_sub(1);
        }
        self.authorities.retain(|_, active| *active > 0);
    }

    pub(super) fn active_for(&self, authority: &RequestAuthority) -> usize {
        self.authorities.get(authority).copied().unwrap_or(0)
    }

    pub(super) fn active_connections(&self) -> Vec<(String, usize)> {
        let mut active: Vec<_> = self
            .authorities
            .iter()
            .map(|(authority, count)| (authority.as_str().to_owned(), *count))
            .collect();
        active.sort_by(|left, right| left.0.cmp(&right.0));
        active
    }

    fn next_admissible(&self) -> Option<usize> {
        self.waiters
            .iter()
            .enumerate()
            .filter(|(_, waiter)| self.authority_available(&waiter.authority))
            .min_by_key(|(_, waiter)| (priority_rank(waiter.priority), waiter.sequence))
            .map(|(index, _)| index)
    }

    fn authority_available(&self, authority: &RequestAuthority) -> bool {
        self.authorities.get(authority).copied().unwrap_or(0) < self.limits.per_authority()
    }

    fn claim(&mut self, authority: &RequestAuthority) {
        self.active += 1;
        *self.authorities.entry(authority.clone()).or_default() += 1;
    }
}

fn priority_rank(priority: PreemptionAuthority) -> u8 {
    match priority {
        PreemptionAuthority::PlaybackCritical => 0,
        PreemptionAuthority::Transition => 1,
        PreemptionAuthority::Speculative => 2,
    }
}
