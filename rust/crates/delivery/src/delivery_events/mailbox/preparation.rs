use super::{signal, MailboxReceiver, MailboxSender};
use crate::delivery_events::{
    PlayerPreparationAdmission, PlayerPreparationFollowup, PlayerPreparationIngress,
    PlayerPreparationReport,
};
use std::collections::VecDeque;

mod ledger;
use ledger::PreparationLedger;

const PLAYER_PREPARATION_PENDING_CAPACITY: usize = 32;

#[derive(Debug)]
pub(super) struct PreparationMailbox {
    pending: VecDeque<PlayerPreparationReport>,
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
    ) -> PlayerPreparationIngress {
        if admission.epoch() != self.admission_epoch || !report.is_initial() {
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
        self.commit_initial(ledger, released, report, cutover);
        PlayerPreparationIngress::Accepted
    }

    fn commit_initial(
        &mut self,
        ledger: PreparationLedger,
        released: Option<PlayerPreparationReport>,
        report: PlayerPreparationReport,
        cutover: bool,
    ) {
        if cutover {
            self.pending.clear();
        }
        self.ledger = ledger;
        if let Some(released) = released {
            self.pending.push_back(released);
        }
        self.pending.push_back(report);
    }

    fn insert_followup(&mut self, report: PlayerPreparationFollowup) -> PlayerPreparationIngress {
        let mut ledger = self.ledger.clone();
        let report = match ledger.admit_followup(report) {
            Ok(report) => report,
            Err(denied) => return denied,
        };
        if !Self::has_capacity(self.pending.len(), ledger.active_len(), 1) {
            return PlayerPreparationIngress::Saturated;
        }
        self.ledger = ledger;
        self.pending.push_back(report);
        PlayerPreparationIngress::Accepted
    }

    fn has_capacity(pending: usize, active: usize, added: usize) -> bool {
        pending + active + added <= PLAYER_PREPARATION_PENDING_CAPACITY
    }

    fn pop(&mut self) -> Option<PlayerPreparationReport> {
        self.pending.pop_front()
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

impl MailboxSender {
    pub(crate) fn player_preparation_admission(&self) -> PlayerPreparationAdmission {
        self.lock().preparations.admission()
    }

    pub(crate) fn send_player_preparation_initial(
        &self,
        admission: PlayerPreparationAdmission,
        report: PlayerPreparationReport,
    ) -> PlayerPreparationIngress {
        self.send_preparation(|mailbox| mailbox.insert_initial(admission, report))
    }

    pub(crate) fn send_player_preparation_followup(
        &self,
        report: PlayerPreparationFollowup,
    ) -> PlayerPreparationIngress {
        self.send_preparation(|mailbox| mailbox.insert_followup(report))
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
        match signal(&self.preparation_wake) {
            true => admission,
            false => PlayerPreparationIngress::Closed,
        }
    }
}

impl MailboxReceiver {
    pub(crate) fn try_player_preparation(&mut self) -> Option<PlayerPreparationReport> {
        self.lock().preparations.pop()
    }

    pub(crate) fn has_player_preparation(&self) -> bool {
        !self.lock().preparations.is_empty()
    }
}
