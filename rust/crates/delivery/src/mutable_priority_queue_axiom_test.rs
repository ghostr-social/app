use super::*;

impl MutablePriorityQueue {
    pub(in super::super) fn wanted(&self) -> HashSet<PlannedTransferId> {
        self.wanted.clone()
    }
}
