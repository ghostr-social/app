#[path = "warp_planner_promotion_contract_test/open_body_model_test.rs"]
mod open_body_model;
#[path = "warp_planner_promotion_support.rs"]
pub(super) mod support;
use crate::adaptive::{CompletionTimes, PlannerCommand, PromotionGrant, ResourceCost};
use support::generated_actions;

#[test]
fn promotion_carries_exact_target_grant_and_incremental_cost() {
    let generated = generated_actions(Some(200_000));
    let promotions: Vec<_> = generated
        .actions
        .iter()
        .filter(|action| matches!(action.command, PlannerCommand::Promote { .. }))
        .collect();
    assert_eq!(promotions.len(), 1);
    let promoted = promotions[0];

    let PlannerCommand::Promote {
        post,
        action,
        source,
        grant,
    } = &promoted.command
    else {
        unreachable!();
    };
    assert_eq!(post.as_str(), "p0");
    assert_eq!(action.value(), 17);
    assert_eq!(source, "https://origin.example/media");
    assert_eq!(
        *grant,
        PromotionGrant {
            maximum_bytes: 200_000,
            valid_until_ms: 20_000,
        }
    );
    assert_eq!(
        promoted.node.resources,
        ResourceCost::new(200_000, 200_000, 0, 0)
    );
    assert_eq!(
        promoted.node.authorized_resources(),
        ResourceCost::new(136_000, 136_000, 0, 0)
    );
    assert!(promoted.node.value.cache_gain_micros > 0);
    assert!(promoted.node.request_profile().is_none());
    assert_eq!(
        promoted.node.forecast.completion,
        CompletionTimes::new(508, 2_034, 8_138, 11_190)
    );
}

#[test]
fn latent_grant_without_observed_response_does_not_promote() {
    let generated = generated_actions(None);
    assert!(!generated
        .actions
        .iter()
        .any(|action| matches!(action.command, PlannerCommand::Promote { .. })));
}

#[test]
fn observed_semantic_promotion_can_reuse_the_complete_reservation() {
    let generated = generated_actions(Some(64_000));
    let promoted = generated
        .actions
        .iter()
        .find(|action| matches!(action.command, PlannerCommand::Promote { .. }))
        .expect("semantic promotion");

    assert_eq!(
        promoted.node.resources,
        ResourceCost::new(64_000, 64_000, 0, 0)
    );
    assert_eq!(
        promoted.node.authorized_resources(),
        ResourceCost::default()
    );
}
