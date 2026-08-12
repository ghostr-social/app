use super::support::{active_hosts, planned_queue, transfer_posts};
use crate::mutable_priority_queue::ForegroundSlots;
use ghostr_engine::adaptive::PreemptionAuthority;
use std::collections::HashSet;

#[test]
fn learned_two_slot_base_fills_and_refills_playback_before_seed_lane() {
    assert_eq!(
        fresh_base_two_posts(),
        ["playing-a", "playing-b", "next"].map(str::to_owned)
    );
    assert_eq!(base_two_refill_post(), "playing-b");
}

fn fresh_base_two_posts() -> [String; 3] {
    let mut queue = planned_queue(
        &[
            ("playing-a", PreemptionAuthority::PlaybackCritical),
            ("playing-b", PreemptionAuthority::PlaybackCritical),
            ("playing-c", PreemptionAuthority::PlaybackCritical),
            ("next", PreemptionAuthority::Transition),
        ],
        "shared.example",
    );
    let first = queue
        .pop_for_hosts(&HashSet::new(), slots(0, 2))
        .expect("first");
    let second = queue
        .pop_for_hosts(&shared_host(), slots(1, 2))
        .expect("second");
    let seed = queue
        .pop_for_hosts(&shared_host(), slots(2, 2))
        .expect("seed");
    transfer_posts(&[first, second, seed])
}

fn base_two_refill_post() -> String {
    let mut queue = planned_queue(
        &[
            ("playing-a", PreemptionAuthority::PlaybackCritical),
            ("playing-b", PreemptionAuthority::PlaybackCritical),
            ("next", PreemptionAuthority::Transition),
        ],
        "shared.example",
    );
    let _active_duplicate = queue.pop_for_hosts(&shared_host(), slots(1, 2));
    queue
        .pop_for_hosts(&shared_host(), slots(1, 2))
        .expect("foreground refill")
        .request
        .chunk
        .post
        .as_str()
        .to_owned()
}

fn shared_host() -> HashSet<String> {
    active_hosts("shared.example")
}

fn slots(active: usize, goal: usize) -> ForegroundSlots {
    ForegroundSlots::new(active, goal)
}
