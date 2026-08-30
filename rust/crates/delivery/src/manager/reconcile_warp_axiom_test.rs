use super::*;

pub(crate) use directive::directive_for;

#[test]
fn terminal_race_supersedes_a_stale_cancel_decision() {
    assert_eq!(
        cancel_outcome(CancelCommit::Missing),
        DecisionOutcome::Superseded
    );
}
