use super::{asset, player_claim, CertifiedReadiness, PreparationContext};
use crate::api::delivery_types::FfiPlaybackPreparationAsset;
use ghostr_delivery::delivery_events::PlanEvidence;

pub(super) async fn project(
    context: &PreparationContext,
    evidence: &PlanEvidence,
) -> Vec<FfiPlaybackPreparationAsset> {
    let mut assets = Vec::new();
    for certificate in &evidence.previous_startups {
        let post = certificate.post();
        let readiness = CertifiedReadiness::Ready(
            certificate,
            player_claim(&evidence.player_preparations, post),
        );
        if let Some(asset) = asset::project(context, post, Some(readiness)).await {
            assets.push(asset);
        }
    }
    assets
}
