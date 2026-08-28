use crate::adaptive::axiom_test_support::predicted_ready_gain;
use crate::adaptive::{ActionKind, CandidateSnapshot, MediaLayout, PlayableRange};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

const READY_MS: u64 = 8_000;
const END: u64 = 100;

#[test]
fn a_partial_information_prefix_does_not_claim_readiness() {
    assert_eq!(gain(&[], ByteRange::new(0, 99), false), 0);
}

#[test]
fn an_exact_requested_remainder_closes_the_playable_range() {
    assert_eq!(gain(&[(0, 40)], ByteRange::new(40, END), false), READY_MS);
}

#[test]
fn a_one_byte_gap_prevents_readiness() {
    assert_eq!(gain(&[(0, 40), (60, 99)], ByteRange::new(40, 60), false), 0,);
}

#[test]
fn an_already_ready_range_is_not_credited_again() {
    assert_eq!(gain(&[(0, END)], ByteRange::new(0, 10), false), 0);
}

#[test]
fn overlapping_out_of_order_present_ranges_are_normalized() {
    assert_eq!(
        gain(
            &[(70, END), (20, 50), (0, 30)],
            ByteRange::new(50, 70),
            false,
        ),
        READY_MS,
    );
}

#[test]
fn direct_playback_blocking_suppresses_a_complete_closure() {
    assert_eq!(gain(&[(0, 40)], ByteRange::new(40, END), true), 0);
}

#[test]
fn an_unknown_layout_does_not_credit_its_synthetic_bootstrap() {
    let mut candidate = candidate(&[]);
    candidate.layout = MediaLayout::Unknown;
    candidate.playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, 262_144),
        playable_ms: READY_MS,
    }];
    for action in [
        ActionKind::Prefix(ByteRange::new(0, 262_144)),
        ActionKind::FetchRange(ByteRange::new(0, 262_144)),
    ] {
        assert_eq!(predicted_ready_gain(&candidate, &action, false), 0);
    }
}

fn gain(present: &[(u64, u64)], action: ByteRange, blocked: bool) -> u64 {
    let candidate = candidate(present);
    predicted_ready_gain(&candidate, &ActionKind::FetchRange(action), blocked)
}

fn candidate(present: &[(u64, u64)]) -> CandidateSnapshot {
    let mut candidate = snapshot(1, 20_000_000, 0, 0).candidates.remove(0);
    candidate.present = present
        .iter()
        .map(|(start, end)| ByteRange::new(*start, *end))
        .collect();
    candidate.playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, END),
        playable_ms: READY_MS,
    }];
    candidate
}
