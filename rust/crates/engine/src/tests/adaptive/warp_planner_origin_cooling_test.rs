use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerCommand, PlannerContext, PlannerRetryAvailability,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::{healthy_origin, snapshot};

const PRIMARY: &str = "https://primary.example/media";
const MIRROR: &str = "https://mirror.example/media";

#[test]
fn cooling_blocks_requests_until_the_authoritative_ready_transition() {
    let mut input = snapshot(1, 8_000_000, 0, 0);
    input.observed_at_ms = 1_000;
    input.candidates[0].origins[0] = healthy_origin(PRIMARY, 20_000_000, 10);
    input.candidates[0]
        .origins
        .push(healthy_origin(MIRROR, 10_000_000, 20));
    let post = input.candidates[0].post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input).with_retry_availability(
        &post,
        PlannerRetryAvailability::Cooling {
            eligible_at_ms: 2_000,
        },
    );

    let cooling = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);
    assert_eq!(network_sources(&cooling).count(), 0);

    input.observed_at_ms = 2_000;
    let still_cooling =
        WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);
    assert_eq!(network_sources(&still_cooling).count(), 0);

    let ready_context = PlannerContext::explicitly_unavailable(&input);
    let ready =
        WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &ready_context);
    assert!(network_sources(&ready).any(|source| source == PRIMARY));
}

fn network_sources(actions: &crate::adaptive::GeneratedActions) -> impl Iterator<Item = &str> {
    actions
        .actions
        .iter()
        .filter_map(|action| match &action.command {
            PlannerCommand::ProbeHead { source, .. } => Some(source.as_str()),
            PlannerCommand::Transfer(transfer) | PlannerCommand::Hedge { transfer, .. } => {
                Some(transfer.source.as_str())
            }
            _ => None,
        })
}
