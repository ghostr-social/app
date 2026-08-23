use super::{
    ClearRequest, CommandReceiver, DeliveryCandidate, DeliveryCommand, DeliveryHandle,
    MailboxReceiver,
};
use crate::evaluation::EvaluationLedger;
use crate::playback_admission::PlaybackAdmission;
use ghostr_engine::PostId;
use tokio::sync::{mpsc, oneshot};

impl DeliveryHandle {
    pub async fn clear(&self) -> anyhow::Result<()> {
        let (reply, result) = oneshot::channel();
        self.clears
            .send(reply)
            .await
            .map_err(|_| anyhow::anyhow!("delivery manager is unavailable"))?;
        result
            .await
            .map_err(|_| anyhow::anyhow!("delivery reset was interrupted"))?
    }
}

impl CommandReceiver {
    pub(crate) fn evaluation(&self) -> EvaluationLedger {
        self.evaluation.clone()
    }

    pub fn receivers(&mut self) -> (&mut MailboxReceiver, &mut mpsc::Receiver<ClearRequest>) {
        (&mut self.commands, &mut self.clears)
    }

    pub(crate) fn discard_pending(&mut self) {
        self.commands.clear();
    }

    pub(crate) fn try_clear(&mut self) -> Option<ClearRequest> {
        self.clears.try_recv().ok()
    }

    pub(crate) fn has_control(&self) -> bool {
        self.commands.has_control()
    }

    pub(crate) fn has_candidate(&self) -> bool {
        self.commands.has_candidate()
    }

    pub fn try_control(&mut self) -> Option<DeliveryCommand> {
        self.commands.try_control()
    }

    pub(crate) fn try_controls_through_focus(&mut self) -> Option<Vec<DeliveryCommand>> {
        self.commands.try_controls_through_focus()
    }

    pub fn try_candidate(&mut self) -> Option<DeliveryCandidate> {
        self.commands.try_candidate()
    }

    pub(crate) fn record_playback_admission(&self, admission: PlaybackAdmission, post: &PostId) {
        self.playback_admissions.record(admission, post);
    }
}
