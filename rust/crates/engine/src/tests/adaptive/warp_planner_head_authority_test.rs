use super::generation::generated_actions;
use crate::adaptive::PlannerCommand;

#[test]
fn a_head_probe_uses_the_same_preemption_authority_as_its_candidate_transfer() {
    let generated = generated_actions();
    let head = generated
        .actions
        .iter()
        .find_map(|action| match &action.command {
            PlannerCommand::ProbeHead { authority, .. } => Some(*authority),
            _ => None,
        });
    let transfer = generated
        .actions
        .iter()
        .find_map(|action| match &action.command {
            PlannerCommand::Transfer(value) => Some(value.authority),
            _ => None,
        });

    assert_eq!(
        head.expect("generated HEAD"),
        transfer.expect("generated transfer")
    );
}
