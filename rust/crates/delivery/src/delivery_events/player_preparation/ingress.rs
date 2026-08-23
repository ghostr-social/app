use super::{
    PlayerPreparationAdmission, PlayerPreparationFollowup, PlayerPreparationIngress,
    PlayerPreparationReport,
};
use crate::delivery_events::{CommandReceiver, DeliveryHandle};

impl DeliveryHandle {
    pub fn player_preparation_admission(&self) -> PlayerPreparationAdmission {
        self.sender.player_preparation_admission()
    }

    pub fn report_player_preparation(
        &self,
        report: PlayerPreparationReport,
    ) -> PlayerPreparationIngress {
        if report.is_initial() {
            let admission = self.player_preparation_admission();
            return self.report_player_preparation_initial(admission, report);
        }
        self.report_player_preparation_followup(PlayerPreparationFollowup::from_report(report))
    }

    pub fn report_player_preparation_initial(
        &self,
        admission: PlayerPreparationAdmission,
        report: PlayerPreparationReport,
    ) -> PlayerPreparationIngress {
        self.sender
            .send_player_preparation_initial(admission, report)
    }

    pub fn report_player_preparation_followup(
        &self,
        report: PlayerPreparationFollowup,
    ) -> PlayerPreparationIngress {
        self.sender.send_player_preparation_followup(report)
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
