use crate::manager::retry::{RetryBook, RetryPolicy};
use ghostr_engine::PostId;

#[test]
fn cooldown_deadline_is_metadata_until_the_timer_completes() {
    let post = PostId::new("post");
    let mut retry = RetryBook::new(RetryPolicy::default());
    let cooldown = retry
        .cool_down_until(post.clone(), 2_000)
        .expect("cooldown");

    assert_eq!(retry.cooling_until(&post), Some(2_000));
    assert!(retry.is_cooling(&post));
    assert!(retry.warm_up(&post, cooldown));
    assert!(!retry.is_cooling(&post));
}
