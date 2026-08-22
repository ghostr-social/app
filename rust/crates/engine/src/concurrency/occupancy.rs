/// Transfer occupancy measured against the capacity the scheduler admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcurrencyOccupancy {
    active_transfers: usize,
    claimed_requests: usize,
    admitted_capacity: usize,
}

impl ConcurrencyOccupancy {
    pub fn new(active_transfers: usize, admitted_capacity: usize) -> Self {
        Self {
            active_transfers,
            claimed_requests: active_transfers,
            admitted_capacity: admitted_capacity.max(1),
        }
    }

    pub fn with_claimed_requests(mut self, claimed_requests: usize) -> Self {
        self.claimed_requests = claimed_requests;
        self
    }

    pub(super) fn fills(self, base_limit: usize) -> bool {
        self.admitted_capacity >= base_limit && self.active_transfers == self.admitted_capacity
    }

    pub(super) fn fills_trial(self, trial_limit: usize) -> bool {
        self.active_transfers >= trial_limit
    }

    pub(super) fn claims(self, trial_limit: usize) -> bool {
        self.claimed_requests >= trial_limit
    }
}
