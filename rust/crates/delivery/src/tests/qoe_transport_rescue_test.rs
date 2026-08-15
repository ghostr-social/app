use crate::delivery_events::{FocusTransition, TransportRescue, TransportRescueReason};
use crate::qoe::QoeTracker;
use ghostr_engine::PostId;

#[test]
fn transport_rescue_keeps_reason_and_rank_out_of_user_navigation() {
    let mut tracker = QoeTracker::default();
    tracker.focus(
        Some(PostId::new("first")),
        FocusTransition::UserNavigation,
        None,
        0,
    );
    let rescue = TransportRescue {
        reason: TransportRescueReason::GraceExpired,
        rank_displacement: 2,
        wait_ms: 250,
    };

    tracker.focus(
        Some(PostId::new("ready")),
        FocusTransition::TransportRescue,
        Some(&rescue),
        250,
    );

    let stats = tracker.stats();
    assert_eq!(stats.user_navigations, 1);
    assert_eq!(stats.transport_substitutions, 1);
    assert_eq!(stats.rank_displacement_total, 2);
    assert_eq!(stats.rescue_wait_total_ms, 250);
    assert_eq!(stats.grace_expired_rescues, 1);
}
