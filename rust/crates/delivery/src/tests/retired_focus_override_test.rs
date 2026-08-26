use crate::manager::failure::FailureClass;
use crate::manager::retry::{Retry, RetryBook, RetryPolicy, Source};
use ghostr_engine::PostId;

#[test]
fn new_focus_revives_only_the_focused_posts_retired_sources() {
    let focused = PostId::new("focused");
    let unrelated = PostId::new("unrelated");
    let focused_source = Source::new(focused.clone(), "https://focused.test/v");
    let unrelated_source = Source::new(unrelated.clone(), "https://other.test/v");
    let mut retry = one_attempt_book();
    retire(&mut retry, focused_source.clone());
    retire(&mut retry, unrelated_source.clone());

    retry.focus_changed(None, Some(&focused));

    assert!(!retry.is_retired(&focused_source));
    assert!(retry.is_retired(&unrelated_source));
}

#[test]
fn repeated_same_focus_does_not_revive_a_retired_source() {
    let focused = PostId::new("focused");
    let source = Source::new(focused.clone(), "https://focused.test/v");
    let mut retry = one_attempt_book();
    retire(&mut retry, source.clone());

    retry.focus_changed(Some(&focused), Some(&focused));

    assert!(retry.is_retired(&source));
}

#[test]
fn new_focus_does_not_revive_a_permanently_failed_source() {
    let focused = PostId::new("focused");
    let source = Source::new(focused.clone(), "https://missing.test/v");
    let mut retry = one_attempt_book();
    while retry.note_failure(source.clone(), FailureClass::Permanent) != Retry::GiveUp {}

    retry.focus_changed(None, Some(&focused));
    retry.focus_changed(Some(&focused), None);
    retry.focus_changed(None, Some(&focused));

    assert!(retry.is_retired(&source));
}

fn one_attempt_book() -> RetryBook {
    RetryBook::new(RetryPolicy {
        transient_attempts: 1,
        ..RetryPolicy::default()
    })
}

fn retire(retry: &mut RetryBook, source: Source) {
    assert_eq!(
        retry.note_failure(source, FailureClass::Transient),
        Retry::GiveUp
    );
}
