use crate::manager::retry::{RetryBook, RetryPolicy, Source};
use ghostr_engine::PostId;

#[test]
fn long_video_demand_tracking_stays_constant_per_focus() {
    let post = PostId::new("long-video");
    let mut retry = RetryBook::new(RetryPolicy::default());
    retry.focus_changed(None, Some(&post));

    for offset in 0..10_000 {
        assert!(retry.expedite_demand(&post, offset));
    }

    assert_eq!(retry.demand_tracking_units(), 32);
}

#[test]
fn success_and_focus_departure_release_demand_tracking() {
    let post = PostId::new("focused");
    let mut retry = RetryBook::new(RetryPolicy::default());
    assert!(retry.expedite_demand(&post, 24));
    assert!(retry.cool_down(post.clone()).is_none());
    assert!(retry.cool_down(post.clone()).is_some());

    retry.note_success(&Source::new(post.clone(), "https://media".into()));
    assert_eq!(retry.demand_tracking_units(), 0);
    assert!(!retry.is_cooling(&post));
    assert!(retry.expedite_demand(&post, 24));

    retry.focus_changed(Some(&post), None);
    assert_eq!(retry.demand_tracking_units(), 0);
    retry.focus_changed(None, Some(&post));
    assert!(retry.expedite_demand(&post, 24));
}

#[test]
fn clear_releases_all_cooldown_state() {
    let post = PostId::new("focused");
    let mut retry = RetryBook::new(RetryPolicy::default());
    assert!(retry.expedite_demand(&post, 24));
    assert!(retry.cool_down(post.clone()).is_none());
    assert!(retry.cool_down(post.clone()).is_some());

    retry.clear();

    assert!(!retry.is_cooling(&post));
    assert_eq!(retry.demand_tracking_units(), 0);
}
