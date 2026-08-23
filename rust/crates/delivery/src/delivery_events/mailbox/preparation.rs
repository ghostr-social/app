use crate::delivery_events::{
    PlayerPreparationActorOutcome, PlayerPreparationAdmission, PlayerPreparationDisposition,
    PlayerPreparationFollowup, PlayerPreparationIngress, PlayerPreparationReport,
};
use std::collections::VecDeque;
use tokio::sync::oneshot;

mod channel;
mod envelope;
mod ledger;
mod receipt;
pub(crate) use envelope::PlayerPreparationEnvelope;
use ledger::PreparationLedger;

const PLAYER_PREPARATION_PENDING_CAPACITY: usize = 32;

#[derive(Debug)]
pub(super) struct PreparationMailbox {
    pending: VecDeque<PlayerPreparationEnvelope>,
    ledger: PreparationLedger,
    admission_epoch: u64,
}

impl Default for PreparationMailbox {
    fn default() -> Self {
        Self {
            pending: VecDeque::new(),
            ledger: PreparationLedger::default(),
            admission_epoch: 1,
        }
    }
}

impl PreparationMailbox {
    fn admission(&self) -> PlayerPreparationAdmission {
        PlayerPreparationAdmission::new(self.admission_epoch)
    }

    fn insert_initial(
        &mut self,
        admission: PlayerPreparationAdmission,
        report: PlayerPreparationReport,
        completion: Option<oneshot::Sender<PlayerPreparationDisposition>>,
    ) -> PlayerPreparationIngress {
        if admission.epoch() != self.admission_epoch {
            return PlayerPreparationIngress::InvalidAdmission;
        }
        if !report.is_initial() {
            return PlayerPreparationIngress::Rejected;
        }
        let cutover = report.client_epoch() > self.ledger.latest_client_epoch();
        let mut ledger = self.ledger.clone();
        let released = match ledger.admit_initial(&report) {
            Ok(released) => released,
            Err(denied) => return denied,
        };
        let pending = if cutover { 0 } else { self.pending.len() };
        let added = usize::from(released.is_some()) + 1;
        if !Self::has_capacity(pending, ledger.active_len(), added) {
            return PlayerPreparationIngress::Saturated;
        }
        self.commit_initial(ledger, released, report, cutover, completion);
        PlayerPreparationIngress::Accepted
    }

    fn commit_initial(
        &mut self,
        ledger: PreparationLedger,
        released: Option<PlayerPreparationReport>,
        report: PlayerPreparationReport,
        cutover: bool,
        completion: Option<oneshot::Sender<PlayerPreparationDisposition>>,
    ) {
        if cutover {
            self.pending.clear();
        }
        self.ledger = ledger;
        if let Some(released) = released {
            self.pending
                .push_back(PlayerPreparationEnvelope::new(released, None));
        }
        self.pending
            .push_back(PlayerPreparationEnvelope::new(report, completion));
    }

    fn insert_followup(
        &mut self,
        report: PlayerPreparationFollowup,
        completion: Option<oneshot::Sender<PlayerPreparationDisposition>>,
    ) -> PlayerPreparationIngress {
        let mut ledger = self.ledger.clone();
        let report = match ledger.admit_followup(report) {
            Ok(report) => report,
            Err(denied) => return denied,
        };
        if !Self::has_capacity(self.pending.len(), ledger.active_len(), 1) {
            return PlayerPreparationIngress::Saturated;
        }
        self.ledger = ledger;
        self.pending
            .push_back(PlayerPreparationEnvelope::new(report, completion));
        PlayerPreparationIngress::Accepted
    }

    fn has_capacity(pending: usize, active: usize, added: usize) -> bool {
        pending + active + added <= PLAYER_PREPARATION_PENDING_CAPACITY
    }

    fn pop(&mut self) -> Option<PlayerPreparationEnvelope> {
        self.pending.pop_front()
    }

    fn probe(&self, report: &PlayerPreparationFollowup) -> Option<PlayerPreparationDisposition> {
        self.ledger.probe(report)
    }

    fn complete(
        &mut self,
        report: &PlayerPreparationReport,
        outcome: PlayerPreparationActorOutcome,
    ) -> PlayerPreparationDisposition {
        let disposition = self.ledger.complete(report, outcome);
        if matches!(
            disposition,
            PlayerPreparationDisposition::Stale | PlayerPreparationDisposition::Rejected
        ) {
            self.settle_attempt(report, disposition);
        }
        disposition
    }

    fn settle_attempt(
        &mut self,
        report: &PlayerPreparationReport,
        disposition: PlayerPreparationDisposition,
    ) {
        let mut retained = VecDeque::new();
        while let Some(envelope) = self.pending.pop_front() {
            if !envelope.report().same_attempt_identity(report) {
                retained.push_back(envelope);
                continue;
            }
            self.ledger.settle_pending(envelope.report(), disposition);
            envelope.complete(disposition);
        }
        self.pending = retained;
    }

    pub(super) fn clear(&mut self) {
        self.pending.clear();
        self.ledger.clear();
        self.admission_epoch = self.admission_epoch.saturating_add(1);
    }

    fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}
