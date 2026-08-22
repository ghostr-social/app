use super::super::decision_record_warp_test_support::{decision, record};
use crate::adaptive::{ActionKind, DecisionReplayStatus, PlannerCommand};
use crate::{ActionId, PostId};

#[test]
fn integrity_valid_but_contradictory_warp_traces_fail_closed() {
    let original = decision(
        "secret-post",
        PlannerCommand::ProbeHead {
            post: PostId::new("secret-post"),
            source: "https://origin.example/media".into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        },
        ActionKind::Head,
    );

    let mut wrong_seed = original.clone();
    wrong_seed.evaluation.as_mut().unwrap().common_random_seed = 1;
    assert_mismatch(&wrong_seed);

    let mut wrong_plan = original.clone();
    wrong_plan
        .search
        .chosen_plan
        .as_mut()
        .unwrap()
        .action_ids
        .clear();
    assert_mismatch(&wrong_plan);

    let mut wrong_command = original;
    wrong_command.selected.as_mut().unwrap().node.kind = ActionKind::Cancel(ActionId::new(3));
    wrong_command.generated.actions[0] = wrong_command.selected.clone().unwrap();
    assert_mismatch(&wrong_command);
}

fn assert_mismatch(decision: &crate::adaptive::WarpPlanningDecision) {
    let captured = record(decision);
    assert_eq!(captured.replay(), DecisionReplayStatus::PlanMismatch);
    assert_eq!(
        captured.replay_warp(),
        Err(DecisionReplayStatus::PlanMismatch)
    );
}
