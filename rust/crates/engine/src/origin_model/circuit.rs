use super::keys::OriginMethodKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const FAILURE_THRESHOLD: u8 = 3;
const INITIAL_BACKOFF_MS: u64 = 2_000;
const MAX_BACKOFF_MS: u64 = 300_000;
const PROBE_LEASE_MS: u64 = 30_000;
const CIRCUIT_CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CircuitStatus {
    Closed,
    Open,
    Recovery,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct CircuitBreaker {
    failures: u8,
    backoff_level: u8,
    retry_at_ms: u64,
    probe_lease_until_ms: u64,
    last_at_ms: u64,
}

impl CircuitBreaker {
    pub fn observe(&mut self, success: bool, at_ms: u64) {
        if success {
            *self = Self::default();
            self.last_at_ms = at_ms;
            return;
        }
        self.last_at_ms = at_ms;
        self.failures = self.failures.saturating_add(1);
        if self.failures < FAILURE_THRESHOLD {
            return;
        }
        let factor = 1u64 << self.backoff_level.min(16);
        let backoff = INITIAL_BACKOFF_MS
            .saturating_mul(factor)
            .min(MAX_BACKOFF_MS);
        self.retry_at_ms = at_ms.saturating_add(backoff);
        self.probe_lease_until_ms = 0;
        self.backoff_level = self.backoff_level.saturating_add(1);
    }

    pub fn status(self, at_ms: u64) -> CircuitStatus {
        if self.failures < FAILURE_THRESHOLD {
            return CircuitStatus::Closed;
        }
        if at_ms < self.retry_at_ms || at_ms < self.probe_lease_until_ms {
            return CircuitStatus::Open;
        }
        CircuitStatus::Recovery
    }

    pub fn claim_recovery(&mut self, at_ms: u64) -> bool {
        if self.status(at_ms) != CircuitStatus::Recovery {
            return false;
        }
        self.probe_lease_until_ms = at_ms.saturating_add(PROBE_LEASE_MS);
        true
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct CircuitBook {
    #[serde(default, with = "super::map_serde")]
    breakers: BTreeMap<OriginMethodKey, CircuitBreaker>,
}

impl CircuitBook {
    pub fn observe(&mut self, key: OriginMethodKey, success: bool, at_ms: u64) {
        self.breakers
            .entry(key)
            .or_default()
            .observe(success, at_ms);
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

    pub fn claim(&mut self, key: OriginMethodKey, at_ms: u64) -> bool {
        self.breakers.entry(key).or_default().claim_recovery(at_ms)
    }

    pub fn normalize_loaded(&mut self) {
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
}
