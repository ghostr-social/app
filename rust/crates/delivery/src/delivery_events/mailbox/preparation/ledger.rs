use crate::delivery_events::{
    PlayerPreparationActorOutcome, PlayerPreparationDisposition, PlayerPreparationFollowup,
    PlayerPreparationIngress, PlayerPreparationReport,
};
use ghostr_engine::PostId;
use std::collections::{HashMap, VecDeque};

use super::receipt::{ReceiptBook, ReceiptProbe};
use fence::AttemptFence;

mod fence;
mod rules;

const ACTIVE_ATTEMPT_CAPACITY: usize = 16;
const ATTEMPT_FENCE_CAPACITY: usize = 16;

#[derive(Clone, Debug, Default)]
pub(super) struct PreparationLedger {
    active: HashMap<PostId, PlayerPreparationReport>,
    fences: VecDeque<AttemptFence>,
    receipts: ReceiptBook,
    latest_client_epoch: u64,
    latest_capability_generation: Option<u64>,
}

impl PreparationLedger {
    pub(super) fn admit_initial(
        &mut self,
        report: &PlayerPreparationReport,
    ) -> Result<Option<PlayerPreparationReport>, PlayerPreparationIngress> {
        self.admit_receipt(report)?;
        self.admit_identity(report)?;
        self.admit_fence(report)?;
        self.admit_initial_capacity(report)?;
        let released = self.release_replaced(report)?;
        self.record_fence(report);
        self.active.insert(report.post().clone(), report.clone());
        self.record_initial_receipt(report, released)
    }

    fn record_initial_receipt(
        &mut self,
        report: &PlayerPreparationReport,
        released: Option<PlayerPreparationReport>,
    ) -> Result<Option<PlayerPreparationReport>, PlayerPreparationIngress> {
        self.receipts
            .record(report, &self.active)
            .then_some(released)
            .ok_or(PlayerPreparationIngress::Saturated)
    }

    pub(super) fn admit_followup(
        &mut self,
        followup: PlayerPreparationFollowup,
    ) -> Result<PlayerPreparationReport, PlayerPreparationIngress> {
        if let Some(probe) = self.receipts.probe_followup(&followup) {
            return Err(ingress(probe));
        }
        let report = self.anchor_followup(followup)?;
        self.track_followup(&report);
        if !self.receipts.record(&report, &self.active) {
            return Err(PlayerPreparationIngress::Saturated);
        }
        Ok(report)
    }

    fn anchor_followup(
        &self,
        followup: PlayerPreparationFollowup,
    ) -> Result<PlayerPreparationReport, PlayerPreparationIngress> {
        let Some(admitted) = self.active.get(followup.post()) else {
            return Err(self.unanchored(&followup));
        };
        let mismatch = self.followup_mismatch(&followup, admitted);
        let report = followup.anchor_to(admitted).ok_or(mismatch)?;
        if report.advances(admitted) {
            return Ok(report);
        }
        Err(if report.sequence() <= admitted.sequence() {
            PlayerPreparationIngress::Stale
        } else {
            PlayerPreparationIngress::Rejected
        })
    }

    fn track_followup(&mut self, report: &PlayerPreparationReport) {
        if report.is_terminal() {
            self.active.remove(report.post());
        } else {
            self.active.insert(report.post().clone(), report.clone());
        }
    }

    pub(super) fn probe(
        &self,
        report: &PlayerPreparationFollowup,
    ) -> Option<PlayerPreparationDisposition> {
        self.receipts.probe_followup(report).map(disposition)
    }

    pub(super) fn complete(
        &mut self,
        report: &PlayerPreparationReport,
        outcome: PlayerPreparationActorOutcome,
    ) -> PlayerPreparationDisposition {
        let outcome = actor_disposition(outcome);
        match self.receipts.complete(report, outcome) {
            None => PlayerPreparationDisposition::Unavailable,
            Some(false) => PlayerPreparationDisposition::Rejected,
            Some(true) => self.reconcile_completion(report, outcome),
        }
    }

    pub(super) fn settle_pending(
        &mut self,
        report: &PlayerPreparationReport,
        outcome: PlayerPreparationDisposition,
    ) {
        if self.receipts.complete(report, outcome) == Some(true) {
            self.reconcile_completion(report, outcome);
        }
    }

    pub(super) fn clear(&mut self) {
        self.active.clear();
        self.fences.clear();
        self.receipts.clear();
        self.latest_client_epoch = 0;
        self.latest_capability_generation = None;
    }

    pub(super) fn active_len(&self) -> usize {
        self.active.len()
    }

    pub(super) fn latest_client_epoch(&self) -> u64 {
        self.latest_client_epoch
    }
}

fn actor_disposition(outcome: PlayerPreparationActorOutcome) -> PlayerPreparationDisposition {
    match outcome {
        PlayerPreparationActorOutcome::Applied => PlayerPreparationDisposition::Applied,
        PlayerPreparationActorOutcome::Stale => PlayerPreparationDisposition::Stale,
        PlayerPreparationActorOutcome::Rejected => PlayerPreparationDisposition::Rejected,
    }
}

fn ingress(probe: ReceiptProbe) -> PlayerPreparationIngress {
    match probe {
        ReceiptProbe::Pending => PlayerPreparationIngress::Pending,
        ReceiptProbe::Final(PlayerPreparationDisposition::Applied)
        | ReceiptProbe::Final(PlayerPreparationDisposition::Duplicate) => {
            PlayerPreparationIngress::Duplicate
        }
        ReceiptProbe::Final(PlayerPreparationDisposition::Stale) => PlayerPreparationIngress::Stale,
        ReceiptProbe::Final(PlayerPreparationDisposition::Rejected) | ReceiptProbe::Conflict => {
            PlayerPreparationIngress::Rejected
        }
        ReceiptProbe::Final(_) => PlayerPreparationIngress::Pending,
    }
}

fn disposition(probe: ReceiptProbe) -> PlayerPreparationDisposition {
    match ingress(probe) {
        PlayerPreparationIngress::Duplicate => PlayerPreparationDisposition::Duplicate,
        PlayerPreparationIngress::Stale => PlayerPreparationDisposition::Stale,
        PlayerPreparationIngress::Rejected => PlayerPreparationDisposition::Rejected,
        _ => PlayerPreparationDisposition::Unavailable,
    }
}
