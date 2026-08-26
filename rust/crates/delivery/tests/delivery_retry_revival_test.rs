//! Giving up is a long cooldown, not an amnesia-proof death sentence:
//! a retired source becomes a candidate again once it has been quiet
//! for the revival window, so a passing network outage cannot silence
//! a host for the rest of the session.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::retry::{cdn_source, retry_policy, CDN_URL};
use ghostr_delivery::manager::failure::FailureClass;
use ghostr_delivery::manager::retry::{Retry, RetryBook};
use ghostr_engine::PostId;

#[tokio::test(start_paused = true)]
async fn delivery_retry_revives_a_retired_source_after_the_long_cooldown() {
    let policy = retry_policy();
    let mut book = RetryBook::new(policy);
    while book.note_failure(cdn_source(), FailureClass::Permanent) != Retry::GiveUp {}
    let post = PostId::new("aa11");
    let urls = vec![CDN_URL.to_owned()];
    assert!(
        book.all_retired(&post, &urls),
        "the source starts out retired"
    );

    tokio::time::sleep(policy.revive_after + Duration::from_secs(1)).await;

    assert!(
        !book.is_retired(&cdn_source()),
        "the revival window expired"
    );
    assert_eq!(book.live_urls(&post, &urls), urls);
}
