//! A source that works again starts from a clean slate: the next
//! failure waits the base delay, not the escalated one.

mod support;

use rust_lib_ghostr::video::delivery_failure::FailureClass;
use rust_lib_ghostr::video::delivery_retry::{Retry, RetryBook};
use support::delivery_retry::{cdn_source, retry_policy};

#[tokio::test]
async fn delivery_retry_backoff_resets_after_a_success() {
    let policy = retry_policy();
    let mut book = RetryBook::new(policy);
    for _ in 0..3 {
        book.note_failure(cdn_source(), FailureClass::Transient);
    }

    book.note_success(&cdn_source());

    let wait = book.note_failure(cdn_source(), FailureClass::Transient);
    assert_eq!(wait, Retry::After(policy.base), "success clears the ladder");
}
