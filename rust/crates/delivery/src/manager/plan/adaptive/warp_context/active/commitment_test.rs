use super::unexpired_nonhistorical_commitment;
use ghostr_engine::adaptive::FeedOffset;

#[test]
fn only_an_unexpired_nonhistorical_commitment_survives_a_temporary_slot_conflict() {
    assert!(unexpired_nonhistorical_commitment(
        FeedOffset::new(0),
        2_000,
        1_000
    ));
    assert!(unexpired_nonhistorical_commitment(
        FeedOffset::new(1),
        2_000,
        1_000
    ));
    assert!(!unexpired_nonhistorical_commitment(
        FeedOffset::new(-2),
        2_000,
        1_000
    ));
    assert!(!unexpired_nonhistorical_commitment(
        FeedOffset::new(1),
        1_000,
        1_000
    ));
}
