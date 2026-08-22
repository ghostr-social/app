use super::generation::generated_actions;
use crate::adaptive::{PlannerCommand, PromotionGrant, ResourceCost};

#[test]
fn promotion_carries_exact_target_grant_and_incremental_cost() {
    let generated = generated_actions();
    let promoted = generated
        .actions
        .iter()
        .find(|action| matches!(action.command, PlannerCommand::Promote { .. }))
        .expect("promotion action");

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
            maximum_bytes: 800_000,
            valid_until_ms: 20_000,
        }
    );
    assert_eq!(
        promoted.node.resources,
        ResourceCost::new(736_000, 736_000, 0, 0)
    );
}
