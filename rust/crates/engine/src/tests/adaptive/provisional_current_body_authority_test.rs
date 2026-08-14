use crate::adaptive::{AdaptivePlayabilityPolicy, CurrentAuthority};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn provisional_current_cannot_schedule_known_media_body_work() {
    let mut input = snapshot(2, 20_000_000, 0, 0);
    input.playback.authority = CurrentAuthority::Provisional;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(
        plan.allocations
            .iter()
            .all(|work| work.post != PostId::new("p0")),
        "{plan:#?}"
    );
}
