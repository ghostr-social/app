mod hard;
pub(in crate::adaptive::warp) use hard::BudgetDenial;
pub use hard::{HardBudget, ResourceCost};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NetworkTokenBucket {
    capacity: u64,
    refill_per_second: u64,
    tokens: u64,
    updated_at_ms: u64,
}

impl NetworkTokenBucket {
    pub const fn new(capacity: u64, refill_per_second: u64, now_ms: u64) -> Self {
        Self {
            capacity,
            refill_per_second,
            tokens: capacity,
            updated_at_ms: now_ms,
        }
    }

    pub fn available(&mut self, now_ms: u64) -> u64 {
        self.refill(now_ms);
        self.tokens
    }

    pub fn consume(&mut self, bytes: u64, now_ms: u64) -> bool {
        self.refill(now_ms);
        if bytes > self.tokens {
            return false;
        }
        self.tokens -= bytes;
        true
    }

    pub fn reconfigure(&mut self, capacity: u64, refill_per_second: u64, now_ms: u64) {
        self.refill(now_ms);
        self.capacity = capacity;
        self.refill_per_second = refill_per_second;
        self.tokens = self.tokens.min(capacity);
    }

    fn refill(&mut self, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.updated_at_ms);
        let added = self.refill_per_second.saturating_mul(elapsed) / 1_000;
        self.tokens = self.tokens.saturating_add(added).min(self.capacity);
        self.updated_at_ms = self.updated_at_ms.max(now_ms);
    }

    pub(crate) const fn replay_parts(&self) -> (u64, u64, u64, u64) {
        (
            self.capacity,
            self.refill_per_second,
            self.tokens,
            self.updated_at_ms,
        )
    }

    pub(crate) const fn from_replay(parts: (u64, u64, u64, u64)) -> Self {
        Self {
            capacity: parts.0,
            refill_per_second: parts.1,
            tokens: parts.2,
            updated_at_ms: parts.3,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceObservation {
    pub network: u64,
    pub storage: u64,
    pub cpu: u64,
    pub requests: u64,
}

impl ResourceObservation {
    pub const fn new(network: u64, storage: u64, cpu: u64, requests: u64) -> Self {
        Self {
            network,
            storage,
            cpu,
            requests,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourcePrices {
    pub network_micros: u64,
    pub storage_micros: u64,
    pub cpu_micros: u64,
    pub request_micros: u64,
}

impl ResourcePrices {
    pub fn cost(self, resources: ResourceCost) -> i64 {
        let value = u128::from(self.network_micros) * u128::from(resources.network_bytes)
            + u128::from(self.storage_micros) * u128::from(resources.storage_bytes)
            + u128::from(self.cpu_micros) * u128::from(resources.cpu_ms)
            + u128::from(self.request_micros) * u128::from(resources.requests);
        value.min(i64::MAX as u128) as i64
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ShadowPriceController {
    prices: ResourcePrices,
}

impl ShadowPriceController {
    pub(crate) const fn from_prices(prices: ResourcePrices) -> Self {
        Self { prices }
    }

    pub fn observe(&mut self, actual: ResourceObservation, target: ResourceObservation) {
        self.prices.network_micros =
            adjust(self.prices.network_micros, actual.network, target.network);
        self.prices.storage_micros =
            adjust(self.prices.storage_micros, actual.storage, target.storage);
        self.prices.cpu_micros = adjust(self.prices.cpu_micros, actual.cpu, target.cpu);
        self.prices.request_micros =
            adjust(self.prices.request_micros, actual.requests, target.requests);
    }

    pub const fn prices(&self) -> ResourcePrices {
        self.prices
    }
}

fn adjust(current: u64, actual: u64, target: u64) -> u64 {
    let target = target.max(1);
    if actual >= target {
        let increase = actual.saturating_sub(target).saturating_mul(1_000) / target;
        return current.saturating_add(increase);
    }
    let decrease = target.saturating_sub(actual).saturating_mul(1_000) / target;
    current.saturating_sub(decrease)
}
