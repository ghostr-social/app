use super::unexpired_current_commitment;
use ghostr_engine::adaptive::FeedOffset;

#[test]
fn only_current_unexpired_nonhistorical_commitments_continue() {
    assert!(unexpired_current_commitment(
        FeedOffset::new(1),
        true,
        2_000,
        1_000
    ));
    assert!(!unexpired_current_commitment(
        FeedOffset::new(1),
        false,
        2_000,
        1_000
    ));
    assert!(!unexpired_current_commitment(
        FeedOffset::new(-1),
        true,
        2_000,
        1_000
    ));
    assert!(!unexpired_current_commitment(
        FeedOffset::new(1),
        true,
        1_000,
        1_000
    ));
}
