use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
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
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ResourceCost {
    pub network_bytes: u64,
    pub storage_bytes: u64,
    pub cpu_ms: u64,
    pub requests: u16,
}

impl ResourceCost {
    pub const fn new(network_bytes: u64, storage_bytes: u64, cpu_ms: u64, requests: u16) -> Self {
        Self {
            network_bytes,
            storage_bytes,
            cpu_ms,
            requests,
        }
    }

    pub fn no_more_than(self, other: Self) -> bool {
        self.network_bytes <= other.network_bytes
            && self.storage_bytes <= other.storage_bytes
            && self.cpu_ms <= other.cpu_ms
            && self.requests <= other.requests
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HardBudget {
    remaining: ResourceCost,
    per_origin_requests: u16,
    origins: BTreeMap<String, u16>,
}

impl HardBudget {
    pub fn new(limits: ResourceCost, per_origin_requests: u16) -> Self {
        Self {
            remaining: limits,
            per_origin_requests,
            origins: BTreeMap::new(),
        }
    }

    pub fn unlimited() -> Self {
        Self::new(
            ResourceCost::new(u64::MAX, u64::MAX, u64::MAX, u16::MAX),
            u16::MAX,
        )
    }

    pub fn consume(&mut self, cost: &ResourceCost, origin: &str) -> bool {
        if !cost.no_more_than(self.remaining) || !self.origin_available(cost.requests, origin) {
            return false;
        }
        self.remaining.network_bytes -= cost.network_bytes;
        self.remaining.storage_bytes -= cost.storage_bytes;
        self.remaining.cpu_ms -= cost.cpu_ms;
        self.remaining.requests -= cost.requests;
        let used = self.origins.entry(origin.to_owned()).or_default();
        *used = used.saturating_add(cost.requests);
        true
    }

    pub fn allows(&self, cost: &ResourceCost, origin: &str) -> bool {
        let mut copy = self.clone();
        copy.consume(cost, origin)
    }

    fn origin_available(&self, requests: u16, origin: &str) -> bool {
        self.origins
            .get(origin)
            .copied()
            .unwrap_or_default()
            .saturating_add(requests)
            <= self.per_origin_requests
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
