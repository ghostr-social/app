#[path = "structural_startup_requirement_support.rs"]
mod support;

use crate::adaptive::{AdaptivePlayabilityPolicy, ControlMode, NextReserveEvidence};
use crate::tests::adaptive_support::snapshot;
use crate::tests::media_timeline_support::{classic_moov, valid_ftyp};
use crate::{ByteRange, PostId};
use support::{candidate, metadata, overlaps};

#[test]
fn adjacent_startup_requires_initialization_and_first_media() {
    let ftyp = valid_ftyp();
    let moov = classic_moov(&[100, 500], &[100, 100]);
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.candidates[0].present = input.candidates[0]
        .playable_ranges
        .iter()
        .take(5)
        .map(|range| range.bytes)
        .collect();

    input.candidates[1] = candidate(&ftyp, &moov, metadata(&moov));
    let metadata_plan = AdaptivePlayabilityPolicy.plan(&input);
    assert!(!matches!(
        metadata_plan.next_reserve,
        NextReserveEvidence::Structural { .. }
    ));

    let movie = ByteRange::new(10_000, 10_000 + moov.len() as u64);
    input.candidates[1] = candidate(
        &ftyp,
        &moov,
        vec![ByteRange::new(0, 24), ByteRange::new(100, 200)],
    );
    let missing_movie = AdaptivePlayabilityPolicy.plan(&input);
    assert!(missing_movie.allocations.iter().any(|work| {
        work.post == PostId::new("p1") && overlaps(work.request.requested_bytes(), movie)
    }));

    let mut complete = metadata(&moov);
    complete.push(ByteRange::new(100, 200));
    input.candidates[1] = candidate(&ftyp, &moov, complete);
    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let NextReserveEvidence::Structural { post, startup } = plan.next_reserve else {
        panic!("exact sparse startup closure was not structural");
    };
    assert_eq!(post, PostId::new("p1"));
    assert_eq!(plan.ready_reserve.ready, 0);
    assert_eq!(plan.ready_reserve.structural, 1);
    assert_eq!(plan.ready_reserve.protected, 1);
    assert_eq!(plan.mode, ControlMode::Emergency);
    assert_eq!(
        startup.ranges(),
        &[
            ByteRange::new(0, 24),
            ByteRange::new(100, 200),
            ByteRange::new(10_000, 10_000 + moov.len() as u64)
        ]
    );
}
