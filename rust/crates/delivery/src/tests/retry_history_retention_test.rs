use crate::manager::failure::FailureClass;
use crate::manager::retry::{Retry, RetryBook, RetryPolicy, Source};
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn evicted_retry_history_is_dropped_without_reviving_retained_sources() {
    let old = PostId::new("old");
    let kept = PostId::new("kept");
    let old_url = url("old");
    let kept_url = url("kept");
    let mut retry = RetryBook::new(RetryPolicy {
        permanent_attempts: 1,
        ..RetryPolicy::default()
    });
    assert_eq!(retire(&mut retry, &old, &old_url), Retry::GiveUp);
    assert_eq!(retire(&mut retry, &kept, &kept_url), Retry::GiveUp);
    retry.cool_down(old.clone()).expect("valid test fixture");
    retry.cool_down(kept.clone()).expect("valid test fixture");

    retry.retain_history(&HashSet::from([kept.clone()]));

    assert!(!retry.is_cooling(&old));
    assert!(retry.is_cooling(&kept));
    assert_eq!(
        retry.live_urls(&old, core::slice::from_ref(&old_url)),
        vec![old_url]
    );
    assert!(retry.live_urls(&kept, &[kept_url]).is_empty());
}

fn retire(retry: &mut RetryBook, post: &PostId, url: &str) -> Retry {
    retry.note_failure(Source::new(post.clone(), url), FailureClass::Permanent)
}

fn url(id: &str) -> String {
    format!("https://media.example/{id}.mp4")
}
