//! A permanent-class failure (DNS, 404, TLS) retires a source after
//! far fewer attempts than a transient one, and a retired source is
//! dropped from the post's candidate list.

mod delivery_fixture;

use delivery_fixture::retry::{cdn_source, retry_policy, CDN_URL};
use ghostr_delivery::delivery_failure::FailureClass;
use ghostr_delivery::delivery_retry::{Retry, RetryBook};
use ghostr_engine::PostId;

fn attempts_until_give_up(class: FailureClass) -> usize {
    let mut book = RetryBook::new(retry_policy());
    (1..=16)
        .find(|_| book.note_failure(cdn_source(), class) == Retry::GiveUp)
        .expect("the policy must give up eventually") as usize
}

#[tokio::test]
async fn delivery_retry_gives_up_on_permanent_failures_far_sooner() {
    let permanent = attempts_until_give_up(FailureClass::Permanent);
    let transient = attempts_until_give_up(FailureClass::Transient);

    assert_eq!(permanent, retry_policy().permanent_attempts as usize);
    assert!(
        permanent < transient,
        "a DNS-class failure must give up sooner than a timeout ({permanent} vs {transient})"
    );
}

#[tokio::test]
async fn delivery_retry_drops_a_retired_source_from_the_candidates() {
    let mut book = RetryBook::new(retry_policy());
    let mirror = "https://mirror.example/video.mp4".to_owned();
    let urls = vec![CDN_URL.to_owned(), mirror.clone()];
    let post = PostId::new("aa11");

    while book.note_failure(cdn_source(), FailureClass::Permanent) != Retry::GiveUp {}

    assert!(book.is_retired(&cdn_source()), "the dead source stays dead");
    assert_eq!(
        book.note_failure(cdn_source(), FailureClass::Permanent),
        Retry::GiveUp,
        "more failures cannot revive a retired source"
    );
    assert_eq!(book.live_urls(&post, &urls), vec![mirror]);
    assert!(!book.all_retired(&post, &urls), "a healthy mirror remains");
}
