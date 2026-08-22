use crate::adaptive::{PlannerCommand, WarpPlanningDecision};
use crate::PostId;

pub(crate) fn add_generated_action(decision: &mut WarpPlanningDecision, id: u16, source: &str) {
    let post = PostId::new(format!("secret-post-{id}"));
    let mut action = decision.generated.actions[0].clone();
    action.node = action.node.with_origin(source);
    action.node.id = id;
    action.node.post = post.clone();
    action.command = PlannerCommand::ProbeHead {
        post,
        source: source.into(),
        authority: crate::adaptive::PreemptionAuthority::Transition,
    };
    decision.generated.actions.push(action);
}
