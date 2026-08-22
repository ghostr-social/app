use super::super::decision_record_warp_test_support::{decision, record};
use crate::adaptive::{ActionKind, DecisionReplayStatus, PlannerCommand};
use crate::PostId;

#[test]
fn an_authentically_truncated_warp_trace_fails_closed() {
    let mut value = decision(
        "secret-post",
        PlannerCommand::ProbeHead {
            post: PostId::new("secret-post"),
            source: "https://origin.example/media".into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        },
        ActionKind::Head,
    );
    value.search.pruned_plan_sample_truncated = true;
    value.search.pruned_plan_events_total = 65;
    let captured = record(&value);

    assert_eq!(
        captured.replay_warp(),
        Err(DecisionReplayStatus::AdvancedReplayUnavailable)
    );
    assert_eq!(
        captured.replay(),
        DecisionReplayStatus::AdvancedReplayUnavailable
    );
}
