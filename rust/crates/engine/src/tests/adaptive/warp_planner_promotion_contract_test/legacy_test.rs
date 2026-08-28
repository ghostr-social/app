use super::support::{legacy_generated_actions, legacy_zero_delta_generated_actions};
use crate::adaptive::PlannerCommand;

#[test]
fn legacy_replay_policy_preserves_historical_latent_promotion_value() {
    let generated = legacy_generated_actions(None);
    let promoted = generated
        .actions
        .iter()
        .find(|action| matches!(action.command, PlannerCommand::Promote { .. }))
        .expect("historical promotion");
    assert_eq!(promoted.node.value.cache_gain_micros, 4_000);
}

#[test]
fn legacy_replay_suppresses_a_zero_delta_promotion() {
    let generated = legacy_zero_delta_generated_actions();
    assert!(!generated
        .actions
        .iter()
        .any(|action| matches!(action.command, PlannerCommand::Promote { .. })));
}
