use super::*;

impl DeliveryHandle {
    #[cfg(test)]
    pub(crate) fn report_player_preparation(
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
