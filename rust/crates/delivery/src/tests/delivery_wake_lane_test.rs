use crate::manager::wake_lane::{WakeCursor, WakeLane};

#[test]
fn continuously_ready_lanes_each_advance_once_per_round() {
    let mut cursor = WakeCursor::default();
    let ready = [true; 9];

    let selected: Vec<_> = (0..18).map(|_| cursor.choose(&ready).unwrap()).collect();

    assert_eq!(
        selected,
        [
            WakeLane::Control,
            WakeLane::PlayerPreparation,
            WakeLane::PlaybackPresentation,
            WakeLane::Candidate,
            WakeLane::Demand,
            WakeLane::Response,
            WakeLane::Internal,
            WakeLane::SegmentedInvalidation,
            WakeLane::Timeline,
            WakeLane::Control,
            WakeLane::PlayerPreparation,
            WakeLane::PlaybackPresentation,
            WakeLane::Candidate,
            WakeLane::Demand,
            WakeLane::Response,
            WakeLane::Internal,
            WakeLane::SegmentedInvalidation,
            WakeLane::Timeline,
        ]
    );
}

#[test]
fn an_awaited_lane_advances_the_same_fairness_cursor() {
    let mut cursor = WakeCursor::default();
    cursor.observe(WakeLane::Demand);

    assert_eq!(
        cursor.choose(&[false, false, false, false, false, false, true, false, false]),
        Some(WakeLane::Internal)
    );
}
