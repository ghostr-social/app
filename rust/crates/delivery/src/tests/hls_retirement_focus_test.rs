use crate::manager::failure::FailureClass;
use crate::manager::retry::{Retry, RetryBook, RetryPolicy, Source};
use ghostr_engine::PostId;

#[test]
fn focus_change_cannot_rearm_a_strict_hls_retirement() {
    let post = PostId::new("hls");
    let source = Source::new(post.clone(), "https://origin.test/root.m3u8".to_owned());
    let mut retry = RetryBook::new(RetryPolicy {
        transient_attempts: 1,
        ..RetryPolicy::default()
    });
    assert_eq!(
        retry.note_hls_failure(source.clone(), FailureClass::Transient),
        Retry::GiveUp
    );

    retry.focus_changed(None, Some(&post));

    assert!(retry.is_retired(&source));
}
