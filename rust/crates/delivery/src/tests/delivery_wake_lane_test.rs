use crate::manager::wake_lane::{WakeCursor, WakeLane};

#[test]
fn continuously_ready_lanes_each_advance_once_per_round() {
    let mut cursor = WakeCursor::default();
    let ready = [true; 4];

    let selected: Vec<_> = (0..8).map(|_| cursor.choose(&ready).unwrap()).collect();

    assert_eq!(
        selected,
        [
            WakeLane::Control,
            WakeLane::Candidate,
            WakeLane::Demand,
            WakeLane::Internal,
            WakeLane::Control,
            WakeLane::Candidate,
            WakeLane::Demand,
            WakeLane::Internal,
        ]
    );
}

#[test]
fn an_awaited_lane_advances_the_same_fairness_cursor() {
    let mut cursor = WakeCursor::default();
    cursor.observe(WakeLane::Demand);

    assert_eq!(
        cursor.choose(&[false, false, true, true]),
        Some(WakeLane::Internal)
    );
}
