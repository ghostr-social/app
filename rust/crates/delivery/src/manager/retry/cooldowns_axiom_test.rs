use super::*;

impl Cooldowns {
    pub(in super::super) fn demand_tracking_units(&self) -> usize {
        self.demanded_offsets.values().map(VecDeque::len).sum()
    }
}
