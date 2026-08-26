use crate::manager::failure::FailureClass;
use crate::manager::retry::{HlsRootAvailability, Retry, RetryBook, RetryPolicy, Source};
use ghostr_engine::PostId;
use core::time::Duration;

#[tokio::test(start_paused = true)]
async fn hls_root_availability_is_atomic_at_retirement_expiry() {
    let post = PostId::new("stream");
    let roots = vec!["primary".to_owned(), "backup".to_owned()];
    let mut retry = RetryBook::new(RetryPolicy {
        transient_attempts: 1,
        revive_after: Duration::from_millis(100),
        ..RetryPolicy::default()
    });
    for root in &roots {
        let source = Source::new(post.clone(), root);
        assert_eq!(retry.note_hls_failure(source, FailureClass::Transient), Retry::GiveUp);
    }

    assert_wait(&retry.hls_root_availability(&post, &roots), 100);
    tokio::time::advance(Duration::from_millis(99)).await;
    assert_wait(&retry.hls_root_availability(&post, &roots), 1);
    tokio::time::advance(Duration::from_millis(1)).await;
    assert_eq!(
        retry.hls_root_availability(&post, &roots),
        HlsRootAvailability::Live(roots)
    );
}

fn assert_wait(availability: &HlsRootAvailability, expected_ms: u64) {
    let HlsRootAvailability::Waiting(wait) = availability else {
        panic!("retired roots must own an exact revival wait");
    };
    assert_eq!(*wait, Duration::from_millis(expected_ms));
}
