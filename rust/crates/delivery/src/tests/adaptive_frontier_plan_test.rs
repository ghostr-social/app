use crate::tests::adaptive_plan_assertions::allocated_posts;
use crate::tests::adaptive_plan_support::plan;
use ghostr_engine::adaptive::StorageSnapshot;

#[test]
fn healthy_network_does_not_expand_the_initial_encoded_reserve() {
    let storage = StorageSnapshot::new(2_000_000_000, 0);
    let poor_posts = allocated_posts(&plan(1_000, 100_000, storage));
    let healthy_posts = allocated_posts(&plan(20_000, 4_000_000, storage));

    assert!(poor_posts.len() <= 2, "{poor_posts:?}");
    assert!(healthy_posts.len() <= 2, "{healthy_posts:?}");
    assert!(healthy_posts.len() >= poor_posts.len());
}
