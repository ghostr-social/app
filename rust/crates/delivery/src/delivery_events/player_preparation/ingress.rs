use super::{
    PlayerPreparationActorOutcome, PlayerPreparationAdmission, PlayerPreparationDisposition,
    PlayerPreparationFollowup, PlayerPreparationIngress, PlayerPreparationReport,
};
use crate::delivery_events::{CommandReceiver, DeliveryHandle, PlayerPreparationEnvelope};
use tokio::sync::oneshot;

impl DeliveryHandle {
    pub fn player_preparation_admission(&self) -> PlayerPreparationAdmission {
        self.sender.player_preparation_admission()
    }

    pub fn player_preparation_disposition(
        &self,
        report: &PlayerPreparationFollowup,
    ) -> Option<PlayerPreparationDisposition> {
        self.sender.player_preparation_disposition(report)
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

    pub async fn confirm_player_preparation_initial(
        &self,
        admission: PlayerPreparationAdmission,
        report: PlayerPreparationReport,
    ) -> PlayerPreparationDisposition {
        let (completion, confirmed) = oneshot::channel();
        let ingress = self.sender.send_player_preparation_initial_with_completion(
            admission,
            report,
            Some(completion),
        );
        confirmed_disposition(ingress, confirmed).await
    }

    pub async fn confirm_player_preparation_followup(
        &self,
        report: PlayerPreparationFollowup,
    ) -> PlayerPreparationDisposition {
        let (completion, confirmed) = oneshot::channel();
        let ingress = self
            .sender
            .send_player_preparation_followup_with_completion(report, Some(completion));
        confirmed_disposition(ingress, confirmed).await
    }
}

impl CommandReceiver {
    pub(crate) fn has_player_preparation(&self) -> bool {
        self.commands.has_player_preparation()
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn try_player_preparation(&mut self) -> Option<PlayerPreparationReport> {
        self.commands.try_player_preparation()
    }

    pub(crate) fn try_player_preparation_envelope(&mut self) -> Option<PlayerPreparationEnvelope> {
        self.commands.try_player_preparation_envelope()
    }

    pub(crate) fn complete_player_preparation(
        &mut self,
        envelope: PlayerPreparationEnvelope,
        outcome: PlayerPreparationActorOutcome,
    ) {
        self.commands.complete_player_preparation(envelope, outcome);
    }
}

async fn confirmed_disposition(
    ingress: PlayerPreparationIngress,
    confirmed: oneshot::Receiver<PlayerPreparationDisposition>,
) -> PlayerPreparationDisposition {
    if ingress == PlayerPreparationIngress::Accepted {
        return confirmed
            .await
            .unwrap_or(PlayerPreparationDisposition::Unavailable);
    }
    immediate_disposition(ingress)
}

fn immediate_disposition(ingress: PlayerPreparationIngress) -> PlayerPreparationDisposition {
    match ingress {
        PlayerPreparationIngress::Duplicate => PlayerPreparationDisposition::Duplicate,
        PlayerPreparationIngress::Stale => PlayerPreparationDisposition::Stale,
        PlayerPreparationIngress::MissingInitial => PlayerPreparationDisposition::MissingInitial,
        _ => terminal_disposition(ingress),
    }
}

fn terminal_disposition(ingress: PlayerPreparationIngress) -> PlayerPreparationDisposition {
    match ingress {
        PlayerPreparationIngress::Rejected => PlayerPreparationDisposition::Rejected,
        PlayerPreparationIngress::InvalidAdmission => PlayerPreparationDisposition::NotAdmitted,
        PlayerPreparationIngress::Saturated => PlayerPreparationDisposition::Saturated,
        PlayerPreparationIngress::Closed => PlayerPreparationDisposition::Closed,
        _ => PlayerPreparationDisposition::Unavailable,
    }
}
