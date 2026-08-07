use crate::scoring::{compare, ChunkRequest};
use crate::tiers::Tier;
use crate::{ByteRange, ChunkId, PostId};
use std::cmp::Ordering;

fn request(post: &str, start: u64, tier: Tier, score: f64) -> ChunkRequest {
    ChunkRequest {
        chunk: ChunkId {
            post: PostId::new(post),
            range: ByteRange::new(start, start + 1),
        },
        tier,
        score,
    }
}

#[test]
fn a_more_urgent_tier_wins_regardless_of_score() {
    let urgent = request("a", 0, Tier::T1CurrentTail, 0.1);
    let valuable = request("b", 0, Tier::T2Startability, 9.9);

    assert_eq!(compare(&urgent, &valuable), Ordering::Less);
}

#[test]
fn a_higher_score_wins_within_a_tier() {
    let strong = request("a", 0, Tier::T2Startability, 2.0);
    let weak = request("b", 0, Tier::T2Startability, 1.0);

    assert_eq!(compare(&strong, &weak), Ordering::Less);
    assert_eq!(compare(&weak, &strong), Ordering::Greater);
}

#[test]
fn equal_scores_break_ties_by_post_id() {
    let first = request("a", 0, Tier::T2Startability, 1.0);
    let second = request("b", 0, Tier::T2Startability, 1.0);

    assert_eq!(compare(&first, &second), Ordering::Less);
}

#[test]
fn equal_posts_break_ties_by_range_start() {
    let early = request("a", 0, Tier::T2Startability, 1.0);
    let late = request("a", 100, Tier::T2Startability, 1.0);

    assert_eq!(compare(&early, &late), Ordering::Less);
}

#[test]
fn sorting_any_permutation_gives_the_same_order() {
    let forward = vec![
        request("b", 0, Tier::T2Startability, 1.0),
        request("a", 5, Tier::T3Deepening, 9.0),
        request("a", 0, Tier::T2Startability, 1.0),
    ];
    let mut left = forward.clone();
    let mut right: Vec<_> = forward.into_iter().rev().collect();

    left.sort_by(compare);
    right.sort_by(compare);

    assert_eq!(left, right);
}
