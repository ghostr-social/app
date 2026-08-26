use super::{PlayerPreparationEnvelope, PreparationMailbox};
use crate::delivery_events::mailbox::{signal, MailboxReceiver, MailboxSender};
use crate::delivery_events::{
    PlayerPreparationActorOutcome, PlayerPreparationAdmission, PlayerPreparationDisposition,
    PlayerPreparationFollowup, PlayerPreparationIngress, PlayerPreparationReport,
};
use tokio::sync::oneshot;

impl MailboxSender {
    pub(crate) fn player_preparation_admission(&self) -> PlayerPreparationAdmission {
        self.lock().preparations.admission()
    }

    pub(crate) fn player_preparation_disposition(
        &self,
        report: &PlayerPreparationFollowup,
    ) -> Option<PlayerPreparationDisposition> {
        if self.preparation_wake.is_closed() {
            return Some(PlayerPreparationDisposition::Closed);
        }
        self.lock().preparations.probe(report)
    }

    pub(crate) fn send_player_preparation_initial_with_completion(
        &self,
        admission: PlayerPreparationAdmission,
        report: PlayerPreparationReport,
        completion: Option<oneshot::Sender<PlayerPreparationDisposition>>,
    ) -> PlayerPreparationIngress {
        self.send_preparation(|mailbox| mailbox.insert_initial(admission, report, completion))
    }

    pub(crate) fn send_player_preparation_followup_with_completion(
        &self,
        report: PlayerPreparationFollowup,
        completion: Option<oneshot::Sender<PlayerPreparationDisposition>>,
    ) -> PlayerPreparationIngress {
        self.send_preparation(|mailbox| mailbox.insert_followup(report, completion))
    }

    fn send_preparation(
        &self,
        insert: impl FnOnce(&mut PreparationMailbox) -> PlayerPreparationIngress,
    ) -> PlayerPreparationIngress {
        if self.preparation_wake.is_closed() {
            return PlayerPreparationIngress::Closed;
        }
        let admission = insert(&mut self.lock().preparations);
        if admission != PlayerPreparationIngress::Accepted {
            return admission;
        }
        if signal(&self.preparation_wake) {
            return admission;
        }
        self.lock().preparations.clear();
        PlayerPreparationIngress::Closed
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "channel/test_support.rs"]
mod test_support;

impl MailboxReceiver {
    #[cfg(any(test, feature = "test"))]
    pub(crate) fn try_player_preparation(&self) -> Option<PlayerPreparationReport> {
        let envelope = self.try_player_preparation_envelope()?;
        let report = envelope.report().clone();
        self.complete_player_preparation(envelope, PlayerPreparationActorOutcome::Applied);
        Some(report)
    }

    pub(crate) fn try_player_preparation_envelope(&self) -> Option<PlayerPreparationEnvelope> {
        self.lock().preparations.pop()
    }

    pub(crate) fn complete_player_preparation(
        &self,
        envelope: PlayerPreparationEnvelope,
        outcome: PlayerPreparationActorOutcome,
    ) {
        let outcome = self
            .lock()
            .preparations
            .complete(envelope.report(), outcome);
        envelope.complete(outcome);
    }

    pub(crate) fn has_player_preparation(&self) -> bool {
        !self.lock().preparations.is_empty()
    }
}
