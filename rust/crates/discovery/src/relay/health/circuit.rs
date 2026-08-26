use super::RelayAdmission;
use super::{INITIAL_BACKOFF, MAX_BACKOFF, PROBE_LEASE};
use std::collections::HashMap;
use tokio::time::Instant;

#[derive(Default)]
pub(super) struct RelayCircuit {
    backoff_level: u8,
    retry_at: Option<Instant>,
    probe_until: Option<Instant>,
    probe_generation: Option<u64>,
    next_generation: u64,
    observed_generation: u64,
    active_generations: HashMap<u64, bool>,
    pub(super) last_at: Option<Instant>,
}

impl RelayCircuit {
    pub(super) fn admit(&mut self, now: Instant, allow_recovery: bool) -> Option<(u64, bool)> {
        let recovery = self.recovery(now, allow_recovery)?;
        self.next_generation = self.next_generation.saturating_add(1);
        self.last_at = Some(now);
        if recovery {
            self.probe_generation = Some(self.next_generation);
        }
        self.active_generations
            .insert(self.next_generation, recovery);
        Some((self.next_generation, recovery))
    }

    pub(super) fn observe(&mut self, generation: u64, success: bool, now: Instant) {
        let Some(recovery) = self.active_generations.remove(&generation) else {
            return;
        };
        if generation < self.observed_generation {
            return;
        }
        self.observed_generation = generation;
        self.last_at = Some(now);
        if success {
            return self.close();
        }
        if self.retry_at.is_some() && !self.matches_probe(generation, recovery) {
            return;
        }
        let delay = backoff(self.backoff_level);
        self.retry_at = Some(now + delay);
        self.probe_until = None;
        self.probe_generation = None;
        self.backoff_level = self.backoff_level.saturating_add(1);
    }

    pub(super) fn release(&mut self, admission: &RelayAdmission, now: Instant) {
        let Some(recovery) = self.active_generations.remove(&admission.generation) else {
            return;
        };
        if recovery && self.probe_generation == Some(admission.generation) {
            self.retry_at = Some(now + backoff(self.backoff_level.saturating_sub(1)));
            self.probe_until = None;
            self.probe_generation = None;
        }
    }

    pub(super) fn recovery_active(&self, now: Instant) -> bool {
        self.probe_until.is_some_and(|until| now < until)
    }

    fn has_in_flight(&self) -> bool {
        !self.active_generations.is_empty()
    }

    pub(super) fn is_cooling(&self) -> bool {
        self.retry_at.is_some()
    }

    pub(super) fn evictable(&self, now: Instant) -> bool {
        !self.has_in_flight() && self.retry_at.is_none_or(|retry_at| retry_at <= now)
    }

    fn recovery(&mut self, now: Instant, allowed: bool) -> Option<bool> {
        let Some(retry_at) = self.retry_at else {
            return Some(false);
        };
        if now < retry_at || self.recovery_active(now) || !allowed {
            return None;
        }
        self.probe_until = Some(now + PROBE_LEASE);
        Some(true)
    }

    fn matches_probe(&self, generation: u64, recovery: bool) -> bool {
        recovery && self.probe_generation == Some(generation)
    }

    fn close(&mut self) {
        self.backoff_level = 0;
        self.retry_at = None;
        self.probe_until = None;
        self.probe_generation = None;
    }
}

fn backoff(level: u8) -> core::time::Duration {
    let factor = 1_u32 << level.min(16);
    INITIAL_BACKOFF.saturating_mul(factor).min(MAX_BACKOFF)
}
