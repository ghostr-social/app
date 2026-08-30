use super::action::{ActiveChunk, HedgeDisposition};
use super::{CompletionStatus, InFlightChunks};
use ghostr_engine::ActionId;

impl InFlightChunks {
    pub(crate) fn authorize_hedge(&self, primary: ActionId) -> bool {
        self.transfers
            .get(&primary)
            .is_some_and(ActiveChunk::authorize_hedge)
    }

    pub(crate) fn release_hedge_authorization(&self, primary: ActionId) {
        if let Some(active) = self.transfers.get(&primary) {
            active.release_hedge();
        }
    }

    pub(crate) fn link_hedge(&mut self, primary: ActionId, alternate: ActionId) -> bool {
        if !self.can_link_hedge(primary, alternate) || !self.ensure_hedge_authorized(primary) {
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

    fn ensure_hedge_authorized(&self, primary: ActionId) -> bool {
        self.transfers
            .get(&primary)
            .is_some_and(|active| active.hedge_authorized() || active.authorize_hedge())
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
    transfers: &std::collections::BTreeMap<ActionId, ActiveChunk>,
    action: ActionId,
) -> Option<HedgeDisposition> {
    let active = transfers.get(&action)?;
    active.hedge_disposition
}
