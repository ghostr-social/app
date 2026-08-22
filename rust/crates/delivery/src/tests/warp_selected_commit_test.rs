use crate::manager::selected_commit::{CommitResult, SelectedCommit};
use ghostr_engine::adaptive::{
    ActionKind, ActionNode, ActionValue, GeneratedAction, PlannerCommand, ResourceCost, WarpPlanner,
};
use ghostr_engine::{ActionId, PostId};

#[test]
fn selected_commit_is_exactly_once_and_rejects_an_excessive_execution() {
    let envelope = ResourceCost::new(80, 60, 4, 1);
    let mut accepted = SelectedCommit::new(action(envelope));
    let mut rejected = SelectedCommit::new(action(envelope));
    let mut planner = WarpPlanner::default();

    assert_eq!(
        accepted.commit(&mut planner, envelope, 1),
        CommitResult::Committed
    );
    assert_eq!(
        accepted.commit(&mut planner, envelope, 1),
        CommitResult::Untracked
    );
    assert_eq!(
        rejected.commit(&mut planner, ResourceCost::new(81, 60, 4, 1), 1),
        CommitResult::Rejected
    );
    assert_eq!(
        rejected.commit(&mut planner, envelope, 1),
        CommitResult::Untracked
    );
}

fn action(resources: ResourceCost) -> GeneratedAction {
    let active = ActionId::new(7);
    GeneratedAction {
        node: ActionNode::new(
            1,
            PostId::new("selected"),
            ActionKind::Cancel(active),
            ActionValue::from_net_micros(0),
        )
        .with_resources(resources),
        command: PlannerCommand::Cancel(active),
    }
}
