use crate::manager::retry::{RetryBook, RetryPolicy};
use ghostr_engine::PostId;

#[test]
fn selected_source_replacement_cancels_its_strict_cooldown() {
    let post = PostId::new("hls");
    let mut retry = RetryBook::new(RetryPolicy::default());
    let stale = retry
        .cool_down_hls_until(post.clone(), 5_000)
        .expect("strict cooldown");

    retry.cancel_hls_cooldown(&post);

    assert!(!retry.is_cooling(&post));
    assert!(!retry.warm_up(&post, stale));
}
