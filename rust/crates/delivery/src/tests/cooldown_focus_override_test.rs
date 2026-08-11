use crate::manager::retry::{RetryBook, RetryPolicy};
use ghostr_engine::PostId;

#[test]
fn stale_timer_cannot_clear_the_cooldown_started_after_focus_override() {
    let post = PostId::new("focused");
    let mut retry = RetryBook::new(RetryPolicy::default());
    let old = retry.cool_down(post.clone()).expect("old cooldown");
    assert!(retry.cool_down(post.clone()).is_none(), "timer stacked");
    retry.focus_changed(None, Some(&post));
    let replacement = retry.cool_down(post.clone()).expect("new cooldown");

    assert!(!retry.warm_up(&post, old), "stale timer was accepted");
    assert!(retry.is_cooling(&post), "replacement cooldown was cleared");
    assert!(retry.warm_up(&post, replacement));
    assert!(!retry.is_cooling(&post));
}

#[test]
fn focus_before_failure_skips_exactly_one_cooldown() {
    let post = PostId::new("focused");
    let mut retry = RetryBook::new(RetryPolicy::default());
    retry.focus_changed(None, Some(&post));

    assert!(retry.cool_down(post.clone()).is_none());
    assert!(!retry.is_cooling(&post));
    assert!(retry.cool_down(post.clone()).is_some());
    assert!(retry.is_cooling(&post));
}

#[test]
fn leaving_focus_revokes_an_unused_retry_credit() {
    let old = PostId::new("old");
    let current = PostId::new("current");
    let mut retry = RetryBook::new(RetryPolicy::default());
    retry.focus_changed(None, Some(&old));

    retry.focus_changed(Some(&old), Some(&current));

    assert!(retry.cool_down(old).is_some(), "old focus kept its credit");
    assert!(
        retry.cool_down(current).is_none(),
        "new focus lost its credit"
    );
}

#[test]
fn repeated_same_focus_does_not_refresh_retry_credit() {
    let post = PostId::new("focused");
    let mut retry = RetryBook::new(RetryPolicy::default());
    retry.focus_changed(None, Some(&post));
    assert!(
        retry.cool_down(post.clone()).is_none(),
        "initial credit missing"
    );
    retry.cool_down(post.clone()).expect("paced retry");

    retry.focus_changed(Some(&post), Some(&post));

    assert!(retry.is_cooling(&post), "same focus cleared the cooldown");
}

#[test]
fn only_a_new_missing_offset_interrupts_the_current_cooldown() {
    let post = PostId::new("playing");
    let mut retry = RetryBook::new(RetryPolicy::default());
    retry.cool_down(post.clone()).expect("first cooldown");
    assert!(retry.expedite_demand(&post, 0));
    retry.cool_down(post.clone()).expect("second cooldown");

    assert!(!retry.expedite_demand(&post, 0));
    assert!(retry.is_cooling(&post), "duplicate demand bypassed backoff");
    assert!(retry.expedite_demand(&post, 8));
    assert!(!retry.is_cooling(&post));
    retry.cool_down(post.clone()).expect("third cooldown");
    assert!(!retry.expedite_demand(&post, 0));
    assert!(retry.is_cooling(&post), "old offset earned credit twice");
    assert!(retry.expedite_demand(&post, 4), "unseen backward seek lost");
    assert!(!retry.is_cooling(&post));
}

#[test]
fn demand_before_failure_skips_exactly_one_cooldown() {
    let post = PostId::new("playing");
    let mut retry = RetryBook::new(RetryPolicy::default());
    assert!(retry.expedite_demand(&post, 8));

    assert!(retry.cool_down(post.clone()).is_none());
    assert!(retry.cool_down(post.clone()).is_some());
    assert!(!retry.expedite_demand(&post, 8));
    assert!(retry.is_cooling(&post));
}
