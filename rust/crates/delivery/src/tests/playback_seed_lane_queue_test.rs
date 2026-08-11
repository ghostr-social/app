use super::support::{active_hosts, planned_queue, transfer_posts};
use crate::mutable_priority_queue::ForegroundSlots;
use ghostr_engine::tiers::Tier;
use std::collections::HashSet;

#[test]
fn playback_frontier_opens_one_protected_startup_lane() {
    for tier in [Tier::T0PlaybackEmergency, Tier::T1CurrentTail] {
        let mut queue = planned_queue(
            &[
                ("playing", tier),
                ("depth", tier),
                ("next", Tier::T2Startability),
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
            ("playing", Tier::T0PlaybackEmergency),
            ("active-seed", Tier::T2Startability),
            ("next-seed", Tier::T2Startability),
        ],
        "shared.example",
    );

    let selected = queue
        .pop_for_hosts(&active_hosts("shared.example"), slots(0, 1))
        .expect("missing playback");

    assert_eq!(selected.request.chunk.post.as_str(), "playing");
}

fn slots(active: usize, goal: usize) -> ForegroundSlots {
    ForegroundSlots::new(active, goal)
}
