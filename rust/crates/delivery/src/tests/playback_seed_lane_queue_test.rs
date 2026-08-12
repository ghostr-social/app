use super::support::{active_hosts, planned_queue, transfer_posts};
use crate::mutable_priority_queue::ForegroundSlots;
use ghostr_engine::adaptive::PreemptionAuthority;
use std::collections::HashSet;

#[test]
fn playback_frontier_opens_one_protected_startup_lane() {
    for authority in [
        PreemptionAuthority::PlaybackCritical,
        PreemptionAuthority::PlaybackCritical,
    ] {
        let mut queue = planned_queue(
            &[
                ("playing", authority),
                ("depth", authority),
                ("next", PreemptionAuthority::Transition),
            ],
            "shared.example",
        );
        let playing = queue
            .pop_for_hosts(&HashSet::new(), slots(0, 1))
            .expect("playing");
        let protected = queue
            .pop_for_hosts(&active_hosts("shared.example"), slots(1, 1))
            .expect("seed");
        assert_eq!(
            transfer_posts(&[playing, protected]),
            ["playing", "next"].map(str::to_owned)
        );
    }
}

#[test]
fn seed_only_replan_restores_missing_playback_before_another_seed() {
    let mut queue = planned_queue(
        &[
            ("playing", PreemptionAuthority::PlaybackCritical),
            ("active-seed", PreemptionAuthority::Transition),
            ("next-seed", PreemptionAuthority::Transition),
        ],
        "shared.example",
    );

    let selected = queue
        .pop_for_hosts(&active_hosts("shared.example"), slots(0, 1))
        .expect("missing playback");

    assert_eq!(selected.request.chunk.post.as_str(), "playing");
}

#[test]
fn a_missing_playback_slot_is_not_loaned_to_another_transition() {
    let mut queue = planned_queue(
        &[
            ("cooling-playback", PreemptionAuthority::PlaybackCritical),
            ("active-seed", PreemptionAuthority::Transition),
            ("next-seed", PreemptionAuthority::Transition),
        ],
        "shared.example",
    );
    let _cooling = queue.pop_for_hosts(&HashSet::new(), slots(0, 1));

    assert!(queue
        .pop_for_hosts(&active_hosts("shared.example"), slots(0, 1))
        .is_none());
}

fn slots(active: usize, goal: usize) -> ForegroundSlots {
    ForegroundSlots::new(active, goal)
}
