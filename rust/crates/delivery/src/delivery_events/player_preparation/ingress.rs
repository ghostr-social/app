use super::{PlayerPreparationIngress, PlayerPreparationReport};
use crate::delivery_events::{CommandReceiver, DeliveryHandle};

impl DeliveryHandle {
    pub fn report_player_preparation(
        &self,
        report: PlayerPreparationReport,
    ) -> PlayerPreparationIngress {
        self.sender.send_player_preparation(report)
    }
}

impl CommandReceiver {
    pub(crate) fn has_player_preparation(&self) -> bool {
        self.commands.has_player_preparation()
    }

    pub fn try_player_preparation(&mut self) -> Option<PlayerPreparationReport> {
        self.commands.try_player_preparation()
    }
}
