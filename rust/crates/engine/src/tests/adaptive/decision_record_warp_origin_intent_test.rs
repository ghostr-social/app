use super::super::decision_record_warp_test_support::{decision, record};
use crate::adaptive::{ActionKind, PlannerCommand};
use crate::origin_model::OriginAdmissionIntent;
use crate::PostId;

#[test]
fn optional_exploration_intent_is_preserved_in_every_action_projection() {
    let mut planned = decision(
        "post",
        PlannerCommand::ProbeHead {
            post: PostId::new("post"),
            source: "https://media.example/video.mp4".into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        },
        ActionKind::Head,
    );
    set_optional(&mut planned);
    let value = serde_json::to_value(record(&planned)).expect("valid record");
    let warp = &value["warp_decision"];

    assert_optional(&warp["selected"]);
    assert_optional(&warp["admissible_actions"][0]);
    assert_optional(&warp["search"]["chosen_plan"]["actions"][0]);
    assert_optional(&warp["search"]["retained_plans"][0]["actions"][0]);
}

fn set_optional(decision: &mut crate::adaptive::WarpPlanningDecision) {
    for action in &mut decision.generated.actions {
        action.node = action
            .node
            .clone()
            .with_origin_admission_intent(OriginAdmissionIntent::OptionalExploration);
    }
    if let Some(action) = decision.selected.as_mut() {
        action.node = action
            .node
            .clone()
            .with_origin_admission_intent(OriginAdmissionIntent::OptionalExploration);
    }
}

fn assert_optional(value: &serde_json::Value) {
    assert_eq!(
        value["origin_admission_intent"],
        serde_json::json!("optional_exploration")
    );
}
