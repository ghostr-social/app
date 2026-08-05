use rust_lib_ghostr::video::partial_range_manifest::RangeManifest;

#[test]
fn range_manifest_rejects_conflicting_bounds_and_handles_empty_states() {
    let mut bounded = RangeManifest::default();
    bounded.set_total_len(8).expect("initial total");
    assert!(bounded.insert(0..9).is_err(), "range exceeds total");
    bounded.insert(0..4).expect("valid range");
    assert!(bounded.set_total_len(3).is_err(), "total truncates bytes");
    assert!(bounded.set_total_len(9).is_err(), "total cannot change");

    let mut unbounded = RangeManifest::default();
    unbounded.insert(0..4).expect("range before total");
    assert!(
        unbounded.set_total_len(3).is_err(),
        "late total truncates bytes"
    );

    let mut empty_range = RangeManifest::default();
    empty_range.insert(4..4).expect("empty range is ignored");
    assert!(empty_range.ranges().is_empty());
    assert!(!empty_range.is_complete(), "unknown total is incomplete");

    let mut zero = RangeManifest::default();
    zero.set_total_len(0).expect("zero total");
    assert!(zero.is_complete(), "zero bytes need no ranges");
}
