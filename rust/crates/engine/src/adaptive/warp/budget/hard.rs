use super::super::request_occupancy::{request_authority, RequestOccupancy};
use std::collections::BTreeMap;

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
    per_origin_requests: usize,
    origins: BTreeMap<String, usize>,
}

impl HardBudget {
    pub fn new(limits: ResourceCost, per_origin_requests: u16) -> Self {
        Self {
            remaining: limits,
            per_origin_requests: usize::from(per_origin_requests),
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
        let authority = request_authority(origin);
        if !cost.no_more_than(self.remaining) || !self.origin_available(cost.requests, &authority) {
            return false;
        }
        self.consume_resources(cost);
        let used = self.origins.entry(authority).or_default();
        *used = used.saturating_add(usize::from(cost.requests));
        true
    }

    pub fn allows(&self, cost: &ResourceCost, origin: &str) -> bool {
        let mut copy = self.clone();
        copy.consume(cost, origin)
    }

    pub(in crate::adaptive::warp) fn with_occupancy(
        mut self,
        occupancy: &RequestOccupancy,
    ) -> Self {
        let active = occupancy.total().min(u16::MAX as usize) as u16;
        self.remaining.requests = self.remaining.requests.saturating_sub(active);
        self.origins = occupancy.authorities().clone();
        self
    }

    fn consume_resources(&mut self, cost: &ResourceCost) {
        self.remaining.network_bytes -= cost.network_bytes;
        self.remaining.storage_bytes -= cost.storage_bytes;
        self.remaining.cpu_ms -= cost.cpu_ms;
        self.remaining.requests -= cost.requests;
    }

    fn origin_available(&self, requests: u16, authority: &str) -> bool {
        self.origins
            .get(authority)
            .copied()
            .unwrap_or_default()
            .saturating_add(usize::from(requests))
            <= self.per_origin_requests
    }
}
