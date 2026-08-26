use super::*;

pub(crate) use channel::command_channel_with_candidate_capacity;

impl DeliveryHandle {
    pub(crate) fn decision_history(&self) -> DecisionHistorySnapshot {
        self.decisions.snapshot()
    }
}
