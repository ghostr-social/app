use crate::manager::retry::{RetryBook, RetryPolicy};
use ghostr_engine::PostId;

#[test]
fn hls_cooldown_cannot_be_bypassed_by_focus_or_playback_demand() {
    let post = PostId::new("playing-hls");
    let previous = PostId::new("previous");
    let mut retry = RetryBook::new(RetryPolicy::default());
    retry.focus_changed(None, Some(&previous));

    let cooldown = retry
        .cool_down_hls_until(post.clone(), 5_000)
        .expect("strict HLS cooldown");
    retry.focus_changed(Some(&previous), Some(&post));
    assert!(retry.is_cooling(&post));
    assert!(!retry.expedite_demand(&post, 8));
    assert!(retry.is_cooling(&post), "demand bypassed HLS backoff");
    assert!(retry.warm_up(&post, cooldown));
    assert!(
        retry.cool_down(post).is_some(),
        "strict wake leaked a credit"
    );
}
