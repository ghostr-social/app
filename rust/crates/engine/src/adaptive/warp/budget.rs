#[cfg(test)]
#[path = "budget/cpu_feedback_test.rs"]
mod cpu_feedback_test;
mod hard;
mod network;
#[cfg(test)]
#[path = "budget/network_refill_test.rs"]
mod network_refill_test;
#[cfg(test)]
#[path = "budget/shadow_price_repeated_update_test.rs"]
mod shadow_price_repeated_update_test;
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
    pub(crate) cpu_micros: u64,
    pub(crate) request_micros: u64,
}

impl ResourcePrices {
    pub(super) fn cost(self, resources: ResourceCost) -> i64 {
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
    pub(super) const fn from_prices(prices: ResourcePrices) -> Self {
        Self { prices }
    }

    pub fn observe(&mut self, actual: ResourceObservation, target: ResourceObservation) {
        self.observe_network(actual.network, target.network);
        self.observe_storage(actual.storage, target.storage);
        self.observe_cpu(actual.cpu, target.cpu);
        self.observe_requests(actual.requests, target.requests);
    }

    pub fn observe_repeated(
        &mut self,
        actual: ResourceObservation,
        target: ResourceObservation,
        count: u128,
    ) {
        self.prices.network_micros = adjust_repeated(
            self.prices.network_micros,
            actual.network,
            target.network,
            count,
        );
        self.prices.storage_micros = adjust_repeated(
            self.prices.storage_micros,
            actual.storage,
            target.storage,
            count,
        );
        self.prices.cpu_micros =
            adjust_cpu_repeated(self.prices.cpu_micros, actual.cpu, target.cpu, count);
        self.prices.request_micros = adjust_repeated(
            self.prices.request_micros,
            actual.requests,
            target.requests,
            count,
        );
    }

    fn observe_network(&mut self, actual: u64, target: u64) {
        self.prices.network_micros = adjust(self.prices.network_micros, actual, target);
    }

    fn observe_storage(&mut self, actual: u64, target: u64) {
        self.prices.storage_micros = adjust(self.prices.storage_micros, actual, target);
    }

    fn observe_cpu(&mut self, actual: u64, target: u64) {
        self.prices.cpu_micros = adjust_observed_cpu(self.prices.cpu_micros, actual, target);
    }

    fn observe_requests(&mut self, actual: u64, target: u64) {
        self.prices.request_micros = adjust(self.prices.request_micros, actual, target);
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
    let (increase, step) = adjustment(actual, target);
    if increase {
        return current.saturating_add(step);
    }
    current.saturating_sub(step)
}

fn adjust_observed_cpu(current: u64, actual: u64, target: u64) -> u64 {
    match target {
        0 => current,
        _ => adjust(current, actual, target),
    }
}

fn adjust_repeated(current: u64, actual: u64, target: u64, count: u128) -> u64 {
    if count == 0 {
        return current;
    }
    let (increase, step) = adjustment(actual, target);
    if increase {
        return saturating_add_repeated(current, step, count);
    }
    saturating_sub_repeated(current, step, count)
}

fn adjustment(actual: u64, target: u64) -> (bool, u64) {
    let target = target.max(1);
    let increasing = actual >= target;
    let difference = if increasing {
        actual - target
    } else {
        target - actual
    };
    let step = u128::from(difference) * 1_000 / u128::from(target);
    (increasing, step.min(u128::from(u64::MAX)) as u64)
}

fn adjust_cpu_repeated(current: u64, actual: u64, target: u64, count: u128) -> u64 {
    match target {
        0 => current,
        _ => adjust_repeated(current, actual, target, count),
    }
}

fn saturating_add_repeated(current: u64, step: u64, count: u128) -> u64 {
    u128::from(current)
        .saturating_add(u128::from(step).saturating_mul(count))
        .min(u128::from(u64::MAX)) as u64
}

fn saturating_sub_repeated(current: u64, step: u64, count: u128) -> u64 {
    let decrease = u128::from(step).saturating_mul(count);
    u128::from(current).saturating_sub(decrease) as u64
}
