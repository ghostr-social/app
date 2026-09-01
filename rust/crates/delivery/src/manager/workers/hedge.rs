use super::DownloadWorkers;
use ghostr_engine::ActionId;

impl DownloadWorkers {
    pub(in crate::manager) fn link_hedge(
        &mut self,
        primary: ActionId,
        alternate: ActionId,
    ) -> bool {
        self.active.link_hedge(primary, alternate)
    }

    pub(in crate::manager) fn authorize_hedge(&self, primary: ActionId) -> bool {
        self.active.authorize_hedge(primary)
    }

    pub(in crate::manager) fn release_hedge_authorization(&self, primary: ActionId) {
        self.active.release_hedge_authorization(primary);
    }

    pub(in crate::manager) fn complete_hedge_winner(&mut self, action: ActionId) -> bool {
        self.active.complete_hedge_winner(action)
    }

    pub(in crate::manager) fn cancel_hedge_loser(&mut self, action: ActionId) -> bool {
        self.active.cancel_hedge_loser(action)
    }
}
