//! Repeated failures against one source back off exponentially and
//! never wait longer than the configured ceiling.

mod support;

use rust_lib_ghostr::video::delivery_failure::FailureClass;
use rust_lib_ghostr::video::delivery_retry::{Retry, RetryBook};
use std::time::Duration;
use support::delivery_retry::{cdn_source, retry_policy};

fn waits(book: &mut RetryBook, attempts: usize) -> Vec<Duration> {
    (0..attempts)
        .map(|_| match book.note_failure(cdn_source(), FailureClass::Transient) {
            Retry::After(wait) => wait,
            Retry::GiveUp => panic!("the policy gave up before the attempt budget ran out"),
        })
        .collect()
}

#[tokio::test]
async fn delivery_retry_backoff_grows_and_stays_bounded() {
    let mut book = RetryBook::new(retry_policy());

    let waits = waits(&mut book, 6);

    let growth = [1, 2, 4, 8].map(Duration::from_secs);
    assert_eq!(waits[..4], growth, "backoff doubles per failed attempt");
    let ceiling = retry_policy().max;
    assert!(
        waits.iter().all(|wait| *wait <= ceiling),
        "backoff must stay bounded by {ceiling:?}: {waits:?}"
    );
}

#[tokio::test]
async fn delivery_retry_backoff_spreads_jittered_waits_around_the_ceiling() {
    let policy = rust_lib_ghostr::video::delivery_retry::RetryPolicy {
        jitter: 0.25,
        ..retry_policy()
    };
    let mut book = RetryBook::new(policy);

    let waits = waits(&mut book, 6);

    let ceiling = policy.max.mul_f64(1.0 + policy.jitter);
    assert!(waits.iter().all(|wait| *wait <= ceiling), "{waits:?}");
    assert!(
        waits.iter().any(|wait| *wait != policy.base),
        "jitter must move waits off the exact base: {waits:?}"
    );
}
