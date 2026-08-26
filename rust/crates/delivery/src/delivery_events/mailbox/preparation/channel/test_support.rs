use super::*;

impl MailboxSender {
    pub(crate) fn send_player_preparation_initial(
        &self,
        admission: PlayerPreparationAdmission,
        report: PlayerPreparationReport,
    ) -> PlayerPreparationIngress {
        self.send_player_preparation_initial_with_completion(admission, report, None)
    }

    pub(crate) fn send_player_preparation_followup(
        &self,
        report: PlayerPreparationFollowup,
    ) -> PlayerPreparationIngress {
        self.send_player_preparation_followup_with_completion(report, None)
    }
}
