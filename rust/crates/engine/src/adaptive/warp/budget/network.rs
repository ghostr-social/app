use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NetworkTokenBucket {
    capacity: u64,
    refill_per_second: u64,
    tokens: u64,
    updated_at_ms: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    refill_milli_bytes: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    debt_bytes: u64,
}

impl NetworkTokenBucket {
    pub(crate) const fn new(capacity: u64, refill_per_second: u64, now_ms: u64) -> Self {
        Self {
            capacity,
            refill_per_second,
            tokens: capacity,
            updated_at_ms: now_ms,
            refill_milli_bytes: 0,
            debt_bytes: 0,
        }
    }

    pub(crate) fn available(&mut self, now_ms: u64) -> u64 {
        self.refill(now_ms);
        self.tokens
    }

    pub(crate) fn consume(&mut self, bytes: u64, now_ms: u64) -> bool {
        self.refill(now_ms);
        if bytes > self.tokens {
            return false;
        }
        self.tokens -= bytes;
        true
    }

    pub(crate) fn reconcile_reservation(&mut self, reserved: u64, actual: u64, now_ms: u64) {
        self.refill(now_ms);
        if actual <= reserved {
            self.credit(reserved - actual);
            return;
        }
        let overrun = actual - reserved;
        let charged = overrun.min(self.tokens);
        self.tokens -= charged;
        self.debt_bytes = self.debt_bytes.saturating_add(overrun - charged);
    }

    pub(crate) fn refill_deadline_ms(&mut self, bytes: u64, now_ms: u64) -> Option<u64> {
        self.refill(now_ms);
        if bytes <= self.tokens || bytes > self.capacity || self.refill_per_second == 0 {
            return None;
        }
        let deficit_bytes = bytes
            .saturating_sub(self.tokens)
            .saturating_add(self.debt_bytes);
        let deficit = u128::from(deficit_bytes).saturating_mul(1_000);
        let deficit = deficit.saturating_sub(u128::from(self.refill_milli_bytes));
        let rate = u128::from(self.refill_per_second);
        let wait_ms = deficit.saturating_add(rate - 1) / rate;
        Some(now_ms.saturating_add(wait_ms.min(u128::from(u64::MAX)) as u64))
    }

    pub(crate) fn reconfigure(&mut self, capacity: u64, refill_per_second: u64, now_ms: u64) {
        self.refill(now_ms);
        let outstanding = self
            .capacity
            .saturating_sub(self.tokens)
            .saturating_add(self.debt_bytes);
        self.capacity = capacity;
        self.refill_per_second = refill_per_second;
        self.tokens = capacity.saturating_sub(outstanding);
        self.debt_bytes = outstanding.saturating_sub(capacity);
        if self.tokens == capacity && self.debt_bytes == 0 {
            self.refill_milli_bytes = 0;
        }
    }

    fn refill(&mut self, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.updated_at_ms);
        let produced = u128::from(self.refill_per_second)
            .saturating_mul(u128::from(elapsed))
            .saturating_add(u128::from(self.refill_milli_bytes));
        let produced_bytes = (produced / 1_000).min(u128::from(u64::MAX)) as u64;
        self.credit(produced_bytes);
        self.refill_milli_bytes = if self.tokens == self.capacity && self.debt_bytes == 0 {
            0
        } else {
            (produced % 1_000) as u64
        };
        self.updated_at_ms = self.updated_at_ms.max(now_ms);
    }

    fn credit(&mut self, bytes: u64) {
        let repaid = bytes.min(self.debt_bytes);
        self.debt_bytes -= repaid;
        self.tokens = self
            .tokens
            .saturating_add(bytes - repaid)
            .min(self.capacity);
    }

    pub(crate) const fn replay_parts(&self) -> (u64, u64, u64, u64, u64, u64) {
        (
            self.capacity,
            self.refill_per_second,
            self.tokens,
            self.updated_at_ms,
            self.refill_milli_bytes,
            self.debt_bytes,
        )
    }

    pub(crate) const fn from_replay(parts: (u64, u64, u64, u64, u64, u64)) -> Self {
        Self {
            capacity: parts.0,
            refill_per_second: parts.1,
            tokens: parts.2,
            updated_at_ms: parts.3,
            refill_milli_bytes: parts.4,
            debt_bytes: parts.5,
        }
    }
}

const fn is_zero(value: &u64) -> bool {
    *value == 0
}
