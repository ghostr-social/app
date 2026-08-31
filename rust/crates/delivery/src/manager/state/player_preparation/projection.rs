use super::DeliveryState;
use crate::segmented::HlsPreparedAssetAuthority;
use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

impl DeliveryState {
    pub(crate) fn player_preparation(
        &self,
        post: &PostId,
        revision: Option<ContentRevision>,
    ) -> PlayerPreparation {
        self.player_preparations
            .get(post)
            .filter(|report| self.player_authority_is_current(report))
            .filter(|report| report.progressive_revision() == revision)
            .map_or(PlayerPreparation::Unverified, |report| {
                report.engine_state()
            })
    }

    pub(crate) fn hls_player_preparation(
        &self,
        authority: &HlsPreparedAssetAuthority,
    ) -> PlayerPreparation {
        self.player_preparations
            .get(authority.post())
            .filter(|report| self.player_authority_is_current(report))
            .filter(|report| report.hls_authority() == Some(authority))
            .map_or(PlayerPreparation::Unverified, |report| {
                report.engine_state()
            })
    }
}
