use super::*;

pub(crate) use channel::command_channel_with_candidate_capacity;
pub(crate) use player_preparation::DECODER_UNSUPPORTED_FAILURE;

impl DeliveryHandle {
    pub(crate) fn decision_history(&self) -> DecisionHistorySnapshot {
        self.decisions.snapshot()
    }
}
