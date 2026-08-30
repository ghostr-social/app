use crate::tests::provisional_handoff_fixture::{detached_next, OBSERVED_AT_MS};
use crate::tests::provisional_handoff_plan_fixture::{generated_cancels, plan_detached};
use std::collections::HashSet;

#[test]
fn detached_handoff_expires_at_and_after_its_commitment_deadline() {
    for deadline in [OBSERVED_AT_MS, OBSERVED_AT_MS - 1] {
        let fixture = detached_next(deadline, None);
        let action = fixture.active.action_id();
        let work = plan_detached(fixture);

        assert!(work.retained.is_empty(), "deadline={deadline}");
        assert_eq!(
            generated_cancels(&work),
            HashSet::from([action]),
            "deadline={deadline}",
        );
    }
}
