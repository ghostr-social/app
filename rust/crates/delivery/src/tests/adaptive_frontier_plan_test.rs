use crate::tests::adaptive_plan_assertions::posts;
use crate::tests::adaptive_plan_support::plan;
use ghostr_engine::adaptive::StorageSnapshot;

#[test]
fn planner_frontier_emerges_from_playback_and_network_evidence_instead_of_four() {
    let storage = StorageSnapshot::new(2_000_000_000, 0);
    let poor_posts = posts(&plan(1_000, 100_000, storage));
    let healthy_posts = posts(&plan(20_000, 4_000_000, storage));

    assert!(poor_posts.len() <= 2, "{poor_posts:?}");
    assert!(healthy_posts.len() > 4, "{healthy_posts:?}");
    assert!(healthy_posts.len() > poor_posts.len());
}
