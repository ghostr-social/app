use super::support::{bind, hedge_record};
use crate::adaptive::DecisionReplayStatus;

#[test]
fn selected_hedge_accepts_the_exact_admitted_alternate_request() {
    let mut record = hedge_record();

    assert!(bind(&mut record));
    assert_eq!(record.chosen_action_id, Some(44));
    assert!(record.executed_request.is_some());
    assert_eq!(record.replay(), DecisionReplayStatus::Verified);
}
