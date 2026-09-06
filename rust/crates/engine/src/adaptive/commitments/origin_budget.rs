use crate::adaptive::{AllocationPlan, PreemptionAuthority, RetainedAllocation};
use crate::RequestAuthority;
use std::collections::HashMap;

pub(super) struct FutureOriginBudget {
    occupied: HashMap<RequestAuthority, usize>,
    limit: usize,
}

impl FutureOriginBudget {
    pub(super) fn new(limit: usize, plan: &AllocationPlan, current: &[RetainedAllocation]) -> Self {
        let occupied = plan
            .allocations
            .iter()
            .filter(|work| work.authority == PreemptionAuthority::PlaybackCritical)
            .filter_map(|work| RequestAuthority::from_url(&work.source))
            .map(|origin| (origin, 1))
            .collect();
        let mut budget = Self { occupied, limit };
        for work in current {
            if let Some(origin) = RequestAuthority::from_url(&work.source) {
                *budget.occupied.entry(origin).or_default() += 1;
            }
        }
        budget
    }

    pub(super) fn admit(&mut self, source: &str) -> bool {
        let Some(origin) = RequestAuthority::from_url(source) else {
            return false;
        };
        let occupied = self.occupied.entry(origin).or_default();
        if *occupied >= self.limit {
            return false;
        }
        *occupied += 1;
        true
    }
}
