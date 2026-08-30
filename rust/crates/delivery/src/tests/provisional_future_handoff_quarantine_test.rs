use crate::tests::provisional_handoff_fixture::{detached_next, NEXT, OBSERVED_AT_MS};
use crate::tests::provisional_handoff_plan_fixture::{generated_cancels, plan_detached};
use ghostr_engine::PostId;
use std::collections::HashSet;

const DIGEST: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

#[test]
fn quarantine_revokes_detached_handoff_retention() {
    let mut fixture = detached_next(4_000, Some(DIGEST));
    let action = fixture.active.action_id();
    let identity = fixture.active.identity().clone();
    let source = identity.source().as_str().to_owned();
    assert_eq!(
        fixture
            .state
            .catalog_mut()
            .quarantine_mirror_group(&identity, DIGEST, OBSERVED_AT_MS),
        vec![PostId::new(NEXT)]
    );
    assert_eq!(
        fixture
            .state
            .catalog()
            .transfer_identity(identity.post(), &source),
        Some(identity.clone())
    );
    assert!(fixture
        .state
        .catalog()
        .deliverable_transfer_identity(identity.post(), &source)
        .is_none());

    let work = plan_detached(fixture);
    assert!(work.retained.is_empty());
    assert_eq!(generated_cancels(&work), HashSet::from([action]));
}
