use crate::delivery_events::{
    PlayerPreparationDisposition, PlayerPreparationFollowup, PlayerPreparationReport,
};
use ghostr_engine::PostId;
use std::collections::{HashMap, VecDeque};

const RECEIPT_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReceiptProbe {
    Pending,
    Final(PlayerPreparationDisposition),
    Conflict,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ReceiptBook {
    receipts: VecDeque<Receipt>,
}

#[derive(Clone, Debug)]
struct Receipt {
    report: PlayerPreparationReport,
    state: ReceiptState,
}

#[derive(Clone, Copy, Debug)]
enum ReceiptState {
    Pending,
    Final(PlayerPreparationDisposition),
}

impl ReceiptBook {
    pub(super) fn probe_report(&self, report: &PlayerPreparationReport) -> Option<ReceiptProbe> {
        let receipt = self
            .receipts
            .iter()
            .find(|receipt| receipt.report.same_receipt_key(report))?;
        Some(receipt.probe(receipt.report == *report))
    }

    pub(super) fn probe_followup(
        &self,
        report: &PlayerPreparationFollowup,
    ) -> Option<ReceiptProbe> {
        let receipt = self
            .receipts
            .iter()
            .find(|receipt| report.same_receipt_key(&receipt.report))?;
        Some(receipt.probe(report.matches_report(&receipt.report)))
    }

    pub(super) fn record(
        &mut self,
        report: &PlayerPreparationReport,
        active: &HashMap<PostId, PlayerPreparationReport>,
    ) -> bool {
        if self.receipts.len() == RECEIPT_CAPACITY && !self.evict_one(active) {
            return false;
        }
        self.receipts.push_back(Receipt {
            report: report.clone(),
            state: ReceiptState::Pending,
        });
        true
    }

    pub(super) fn complete(
        &mut self,
        report: &PlayerPreparationReport,
        disposition: PlayerPreparationDisposition,
    ) -> Option<bool> {
        let receipt = self
            .receipts
            .iter_mut()
            .find(|receipt| receipt.report.same_receipt_key(report))?;
        if receipt.report != *report {
            return Some(false);
        }
        receipt.state = ReceiptState::Final(disposition);
        Some(true)
    }

    pub(super) fn clear(&mut self) {
        self.receipts.clear();
    }

    fn evict_one(&mut self, active: &HashMap<PostId, PlayerPreparationReport>) -> bool {
        let Some(index) = self.receipts.iter().position(|receipt| {
            !receipt.is_pending()
                && !active
                    .values()
                    .any(|report| report.same_attempt_identity(&receipt.report))
        }) else {
            return false;
        };
        self.receipts.remove(index);
        true
    }
}

impl Receipt {
    fn is_pending(&self) -> bool {
        matches!(self.state, ReceiptState::Pending)
    }

    fn probe(&self, exact_payload: bool) -> ReceiptProbe {
        if !exact_payload {
            return ReceiptProbe::Conflict;
        }
        match self.state {
            ReceiptState::Pending => ReceiptProbe::Pending,
            ReceiptState::Final(disposition) => ReceiptProbe::Final(disposition),
        }
    }
}
