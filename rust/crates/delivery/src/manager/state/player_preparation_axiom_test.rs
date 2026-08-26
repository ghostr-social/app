use super::*;

impl DeliveryState {
    pub(crate) fn apply_player_preparation(&mut self, report: PlayerPreparationReport) -> bool {
        self.apply_player_preparation_at(report, 0) == PlayerPreparationActorOutcome::Applied
    }
}
