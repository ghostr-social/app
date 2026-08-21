use super::action::{ActiveChunk, HedgeDisposition};
use super::{CompletionStatus, InFlightChunks};
use ghostr_engine::ActionId;

impl InFlightChunks {
    pub(crate) fn link_hedge(&mut self, primary: ActionId, alternate: ActionId) -> bool {
        if !self.can_link_hedge(primary, alternate) {
            return false;
        }
        self.hedges.insert(primary, alternate);
        self.hedges.insert(alternate, primary);
        true
    }

    pub(crate) fn complete_hedge_winner(&mut self, winner: ActionId) -> bool {
        let Some(loser) = self.hedges.get(&winner).copied() else {
            return false;
        };
        if self.hedge_already_lost(winner, loser) {
            return false;
        }
        let Some(active) = self.transfers.get_mut(&winner) else {
            return false;
        };
        active.hedge_disposition = Some(HedgeDisposition::Winner);
        if let Some(active) = self.transfers.get_mut(&loser) {
            active.hedge_disposition = Some(HedgeDisposition::Loser);
            active.cancel();
        }
        true
    }

    pub(crate) fn cancel_hedge_loser(&mut self, action: ActionId) -> bool {
        let Some(active) = self.transfers.get_mut(&action) else {
            return false;
        };
        active.hedge_disposition = Some(HedgeDisposition::Loser);
        active.cancel();
        true
    }

    fn can_link_hedge(&self, primary: ActionId, alternate: ActionId) -> bool {
        primary != alternate
            && self.transfers.contains_key(&primary)
            && self.transfers.contains_key(&alternate)
            && !self.hedges.contains_key(&primary)
            && !self.hedges.contains_key(&alternate)
    }

    fn hedge_already_lost(&self, winner: ActionId, loser: ActionId) -> bool {
        disposition(&self.transfers, winner) == Some(HedgeDisposition::Loser)
            || disposition(&self.transfers, loser) == Some(HedgeDisposition::Winner)
    }
}

pub(super) fn completion_status(active: &ActiveChunk, linked: bool) -> CompletionStatus {
    match active.hedge_disposition {
        Some(HedgeDisposition::Winner) => CompletionStatus::HedgeWinner,
        Some(HedgeDisposition::Loser) => CompletionStatus::HedgeLoser,
        None if linked => CompletionStatus::HedgeLoser,
        None if active.cancelling => CompletionStatus::Cancelled,
        None => CompletionStatus::Current,
    }
}

fn disposition(
    transfers: &std::collections::HashMap<ActionId, ActiveChunk>,
    action: ActionId,
) -> Option<HedgeDisposition> {
    transfers
        .get(&action)
        .and_then(|active| active.hedge_disposition)
}
