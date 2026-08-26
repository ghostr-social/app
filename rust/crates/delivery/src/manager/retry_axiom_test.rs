use super::*;

impl RetryBook {
    pub(crate) fn demand_tracking_units(&self) -> usize {
        self.cooldowns.demand_tracking_units()
    }
}
