use super::super::decision_record_warp_test_support::decision;
use crate::adaptive::{ActionKind, PlannerCommand, WarpPlanningDecision};
use crate::PostId;

pub(super) fn head_decision() -> WarpPlanningDecision {
    decision(
        "secret-post",
        PlannerCommand::ProbeHead {
            post: PostId::new("secret-post"),
            source: "https://origin.example/media".into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        },
        ActionKind::Head,
    )
}
