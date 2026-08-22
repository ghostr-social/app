#[cfg(test)]
#[path = "budget/cpu_feedback_test.rs"]
mod cpu_feedback_test;
mod hard;
mod network;
#[cfg(test)]
#[path = "budget/network_refill_test.rs"]
mod network_refill_test;
pub(in crate::adaptive::warp) use hard::BudgetDenial;
pub use hard::{HardBudget, ResourceCost};
pub use network::NetworkTokenBucket;
use serde::{Deserialize, Serialize};

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
        self.prices.cpu_micros =
            adjust_observed_cpu(self.prices.cpu_micros, actual.cpu, target.cpu);
        self.prices.request_micros =
            adjust(self.prices.request_micros, actual.requests, target.requests);
    }

    pub const fn prices(&self) -> ResourcePrices {
        self.prices
    }

    pub const fn cpu_operating_target_ms(hard_limit_ms: u64) -> u64 {
        if hard_limit_ms <= 1 {
            return hard_limit_ms;
        }
        let whole = hard_limit_ms / 10 * 9;
        let remainder = hard_limit_ms % 10 * 9 / 10;
        let target = whole.saturating_add(remainder);
        if target == 0 {
            1
        } else {
            target
        }
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

fn adjust_observed_cpu(current: u64, actual: u64, target: u64) -> u64 {
    match target {
        0 => current,
        _ => adjust(current, actual, target),
    }
}
