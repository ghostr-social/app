use super::DeliveryState;
use crate::delivery_events::{PlayerPreparationClaim, PlayerPreparationState};
use ghostr_engine::PostId;

impl DeliveryState {
    pub(crate) fn player_preparation_claims(
        &self,
        posts: &[PostId],
    ) -> Vec<PlayerPreparationClaim> {
        posts
            .iter()
            .filter_map(|post| self.player_preparation_claim(post))
            .collect()
    }

    fn player_preparation_claim(&self, post: &PostId) -> Option<PlayerPreparationClaim> {
        let report = self.player_preparations.get(post)?;
        (report.state() == PlayerPreparationState::FirstFrameRendered
            && self.player_authority_is_current(report))
        .then(|| report.claim())
    }
}
