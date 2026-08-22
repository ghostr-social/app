use super::super::decision_record_warp_test_support::allocation;
use super::warp_planner_commit_test_support::{planner, OBSERVED_AT_MS};
use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, GeneratedAction, PlannerCommand, ResourceCost,
    RetrievalRequest,
};
use crate::{ByteRange, PostId};

#[test]
fn progressive_request_gets_an_exact_future_token_deadline() {
    let mut planner = planner(100, 20);
    let action = progressive_action(100);
    assert!(planner.commit(&action, ResourceCost::new(90, 90, 0, 1), OBSERVED_AT_MS,));

    assert_eq!(
        planner.next_network_refill_deadline_ms(&[action], OBSERVED_AT_MS),
        Some(OBSERVED_AT_MS + 4_500),
    );
}

fn progressive_action(bytes: u64) -> GeneratedAction {
    let range = ByteRange::new(0, bytes);
    let request = RetrievalRequest::FetchRange {
        bytes: range,
        promotion: None,
    };
    GeneratedAction {
        node: ActionNode::new(
            1,
            PostId::new("progressive"),
            ActionKind::FetchRange(range),
            ActionValue::default(),
        )
        .with_resources(ResourceCost::new(bytes, bytes, 0, 1)),
        command: PlannerCommand::Transfer(allocation("https://origin.example/video", request)),
    }
}
