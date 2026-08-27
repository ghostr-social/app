use super::keys::OriginMethodKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const FAILURE_THRESHOLD: u8 = 3;
const INITIAL_BACKOFF_MS: u64 = 2_000;
const MAX_BACKOFF_MS: u64 = 300_000;
const PROBE_LEASE_MS: u64 = 30_000;
const CIRCUIT_CAPACITY: usize = 256;

mod recovery;
pub(super) use recovery::{CircuitStatus, RecoveryClaim, RecoveryResolution, RecoveryStage};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct CircuitBreaker {
    failures: u8,
    backoff_level: u8,
    retry_at_ms: u64,
    probe_lease_until_ms: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    probe_generation: u64,
    #[serde(default, skip_serializing_if = "is_false")]
    trial_pending: bool,
    last_at_ms: u64,
}

impl CircuitBreaker {
    pub fn observe(&mut self, success: bool, at_ms: u64) {
        if success {
            self.close(at_ms);
            return;
        }
        self.last_at_ms = at_ms;
        self.failures = self.failures.saturating_add(1);
        if self.failures < FAILURE_THRESHOLD {
            return;
        }
        self.back_off(at_ms, false);
    }

    fn close(&mut self, at_ms: u64) {
        let probe_generation = self.probe_generation;
        *self = Self::default();
        self.probe_generation = probe_generation;
        self.last_at_ms = at_ms;
    }

    fn back_off(&mut self, at_ms: u64, preserve_trial: bool) {
        let factor = 1u64 << self.backoff_level.min(16);
        let backoff = INITIAL_BACKOFF_MS
            .saturating_mul(factor)
            .min(MAX_BACKOFF_MS);
        self.retry_at_ms = at_ms.saturating_add(backoff);
        self.probe_lease_until_ms = 0;
        self.trial_pending &= preserve_trial;
        self.backoff_level = self.backoff_level.saturating_add(1);
    }

    pub(super) fn defer_recovery(&mut self, at_ms: u64) {
        self.last_at_ms = at_ms;
        self.back_off(at_ms, true);
    }

    pub fn status(self, at_ms: u64) -> CircuitStatus {
        if self.failures < FAILURE_THRESHOLD {
            return CircuitStatus::Closed;
        }
        if at_ms < self.retry_at_ms || at_ms < self.probe_lease_until_ms {
            return CircuitStatus::Open;
        }
        if self.trial_pending {
            CircuitStatus::RecoveryTrial
        } else {
            CircuitStatus::RecoveryProbe
        }
    }

    fn normalize_loaded(&mut self) {
        self.probe_lease_until_ms = 0;
    }
}

const fn is_zero(value: &u64) -> bool {
    *value == 0
}

const fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct CircuitBook {
    #[serde(default, with = "super::map_serde")]
    breakers: BTreeMap<OriginMethodKey, CircuitBreaker>,
}

impl CircuitBook {
    pub fn observe(&mut self, key: OriginMethodKey, success: bool, at_ms: u64) {
        let breaker = self.breakers.entry(key).or_default();
        if breaker.status(at_ms) != CircuitStatus::Closed {
            return;
        }
        breaker.observe(success, at_ms);
        while self.breakers.len() > CIRCUIT_CAPACITY {
            let victim = self
                .breakers
                .iter()
                .min_by_key(|(key, value)| (value.last_at_ms, *key))
                .map(|(key, _)| key.clone());
            let Some(victim) = victim else { return };
            self.breakers.remove(&victim);
        }
    }

    pub fn status(&self, key: &OriginMethodKey, at_ms: u64) -> CircuitStatus {
        self.breakers
            .get(key)
            .copied()
            .unwrap_or_default()
            .status(at_ms)
    }

    pub fn claim(&mut self, key: OriginMethodKey, at_ms: u64) -> Option<RecoveryClaim> {
        let (generation, stage) = self
            .breakers
            .entry(key.clone())
            .or_default()
            .claim_recovery(at_ms)?;
        Some(RecoveryClaim {
            key,
            generation,
            stage,
        })
    }

    pub fn complete(&mut self, claim: &RecoveryClaim, resolution: RecoveryResolution) -> bool {
        self.breakers
            .get_mut(&claim.key)
            .is_some_and(|breaker| breaker.complete_recovery(claim, resolution))
    }

    pub fn is_current(&self, claim: &RecoveryClaim) -> bool {
        self.breakers
            .get(&claim.key)
            .is_some_and(|breaker| breaker.matches(claim))
    }

    pub fn normalize_loaded(&mut self) {
        self.breakers
            .values_mut()
            .for_each(CircuitBreaker::normalize_loaded);
        while self.breakers.len() > CIRCUIT_CAPACITY {
            let Some(victim) = self.oldest() else { return };
            self.breakers.remove(&victim);
        }
    }

    fn oldest(&self) -> Option<OriginMethodKey> {
        self.breakers
            .iter()
            .min_by_key(|(key, value)| (value.last_at_ms, *key))
            .map(|(key, _)| key.clone())
    }

    pub(super) fn replay_project(&self, aliases: impl Fn(&str) -> Vec<String>) -> Self {
        let breakers = self
            .breakers
            .iter()
            .flat_map(|(key, value)| {
                aliases(&key.origin).into_iter().map(|origin| {
                    (
                        OriginMethodKey {
                            origin,
                            method: key.method,
                        },
                        *value,
                    )
                })
            })
            .collect();
        Self { breakers }
    }

    pub(super) fn replay_bounded(&self) -> bool {
        self.breakers.len() <= CIRCUIT_CAPACITY
    }
}
