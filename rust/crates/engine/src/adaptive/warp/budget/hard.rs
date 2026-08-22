use super::super::request_occupancy::RequestOccupancy;
use super::super::SegmentedStorageBudget;
use crate::adaptive::ActionNode;
use crate::RequestAuthority;
use std::collections::BTreeMap;

mod reservation;
mod segmented_storage;

#[cfg(test)]
#[path = "hard/tests/mod.rs"]
mod tests;

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
    request_width: u16,
    per_origin_requests: usize,
    origins: BTreeMap<RequestAuthority, usize>,
    pending_rescue: Vec<ActionNode>,
    segmented_storage: Option<SegmentedStorageBudget>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::adaptive::warp) enum BudgetDenial {
    HardLimit,
    RescueReserve,
}

impl HardBudget {
    pub fn new(limits: ResourceCost, per_origin_requests: u16) -> Self {
        Self {
            remaining: limits,
            request_width: limits.requests,
            per_origin_requests: usize::from(per_origin_requests),
            origins: BTreeMap::new(),
            pending_rescue: Vec::new(),
            segmented_storage: None,
        }
    }

    pub fn unlimited() -> Self {
        Self::new(
            ResourceCost::new(u64::MAX, u64::MAX, u64::MAX, u16::MAX),
            u16::MAX,
        )
    }

    pub fn consume(&mut self, cost: &ResourceCost, authority: Option<&RequestAuthority>) -> bool {
        self.consume_raw(cost, authority)
    }

    pub(super) fn consume_raw(
        &mut self,
        cost: &ResourceCost,
        authority: Option<&RequestAuthority>,
    ) -> bool {
        if !cost.no_more_than(self.remaining) {
            return false;
        }
        if cost.requests == 0 {
            self.consume_resources(cost);
            return true;
        }
        let Some(authority) = authority else {
            return false;
        };
        if !self.origin_available(cost.requests, authority) {
            return false;
        }
        self.consume_resources(cost);
        let used = self.origins.entry(authority.clone()).or_default();
        *used = used.saturating_add(usize::from(cost.requests));
        true
    }

    pub fn allows(&self, cost: &ResourceCost, authority: Option<&RequestAuthority>) -> bool {
        let mut copy = self.clone();
        copy.consume(cost, authority)
    }

    pub(crate) const fn replay_remaining(&self) -> ResourceCost {
        self.remaining
    }

    pub(crate) const fn replay_per_origin_requests(&self) -> usize {
        self.per_origin_requests
    }

    pub(crate) const fn replay_request_width(&self) -> u16 {
        self.request_width
    }

    pub(crate) fn replay_origins(&self) -> &BTreeMap<RequestAuthority, usize> {
        &self.origins
    }

    pub(crate) fn replay_pending(&self) -> &[ActionNode] {
        &self.pending_rescue
    }

    pub(crate) const fn replay_segmented_storage(&self) -> Option<SegmentedStorageBudget> {
        self.segmented_storage
    }

    pub(crate) fn from_replay(
        state: (
            ResourceCost,
            u16,
            usize,
            BTreeMap<RequestAuthority, usize>,
            Vec<ActionNode>,
            Option<SegmentedStorageBudget>,
        ),
    ) -> Option<Self> {
        let (remaining, request_width, per_origin_requests, origins, pending_rescue, segmented) =
            state;
        let budget = Self {
            remaining,
            request_width,
            per_origin_requests,
            origins,
            pending_rescue: Vec::new(),
            segmented_storage: segmented,
        };
        budget.protect(&pending_rescue)
    }

    pub(in crate::adaptive::warp) const fn with_segmented_storage(
        mut self,
        budget: SegmentedStorageBudget,
    ) -> Self {
        self.segmented_storage = Some(budget);
        self
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

    fn origin_available(&self, requests: u16, authority: &RequestAuthority) -> bool {
        self.origins
            .get(authority)
            .copied()
            .unwrap_or_default()
            .saturating_add(usize::from(requests))
            <= self.per_origin_requests
    }
}
