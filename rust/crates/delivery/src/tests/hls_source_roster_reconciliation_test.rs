use crate::manager::failure::FailureClass;
use crate::manager::retry::{Retry, RetryBook, RetryPolicy, Source};
use ghostr_engine::PostId;

#[test]
fn roster_reconciliation_preserves_unchanged_retirements() {
    let post = PostId::new("hls");
    let failed = Source::new(post.clone(), "https://a.example/root.m3u8".to_owned());
    let mut retry = RetryBook::new(RetryPolicy {
        permanent_attempts: 1,
        ..RetryPolicy::default()
    });
    assert_eq!(
        retry.note_hls_failure(failed.clone(), FailureClass::Permanent),
        Retry::GiveUp
    );

    retry.reconcile_hls_sources(
        &post,
        &[
            "https://b.example/root.m3u8".to_owned(),
            "https://a.example/root.m3u8".to_owned(),
            "https://c.example/root.m3u8".to_owned(),
        ],
    );

    assert!(retry.is_retired(&failed));
    let removed = Source::new(post.clone(), "https://b.example/root.m3u8".to_owned());
    assert_eq!(
        retry.note_hls_failure(removed.clone(), FailureClass::Permanent),
        Retry::GiveUp
    );
    retry.reconcile_hls_sources(
        &post,
        &[
            "https://a.example/root.m3u8".to_owned(),
            "https://c.example/root.m3u8".to_owned(),
        ],
    );
    assert!(!retry.is_retired(&removed));
}
