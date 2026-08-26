use crate::manager::failure::FailureClass;
use crate::manager::retry::{Retry, RetryBook, RetryPolicy, Source};
use ghostr_engine::PostId;

#[test]
fn a_failed_source_yields_immediately_to_an_untried_mirror() {
    let post = PostId::new("clip");
    let primary = "https://primary.example/clip.mp4".to_owned();
    let mirror = "https://mirror.example/clip.mp4".to_owned();
    let urls = vec![primary.clone(), mirror.clone()];
    let mut retry = RetryBook::new(RetryPolicy::default());

    let decision = retry.note_failure(
        Source::new(post.clone(), &primary),
        FailureClass::Transient,
    );

    assert!(matches!(decision, Retry::After(_)));
    assert!(retry.has_ready_alternative(&post, &primary, &urls));
    assert_eq!(retry.live_urls(&post, &urls), vec![mirror, primary]);
}
