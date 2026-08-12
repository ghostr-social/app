/// Transfer occupancy measured against the capacity the scheduler admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcurrencyOccupancy {
    active_transfers: usize,
    admitted_capacity: usize,
}

impl ConcurrencyOccupancy {
    pub fn new(active_transfers: usize, admitted_capacity: usize) -> Self {
        Self {
            active_transfers,
            admitted_capacity: admitted_capacity.max(1),
        }
    }

    pub(super) fn fills(self, base_limit: usize) -> bool {
        self.admitted_capacity >= base_limit && self.active_transfers == self.admitted_capacity
    }
}
