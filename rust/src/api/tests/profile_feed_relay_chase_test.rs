//! Opening a profile feed starts NIP-65 discovery for its creators.

use crate::api::tests::outbox_runtime_support::test_bootstrap;
use crate::api::tests::runtime_fixture::runtime;
use crate::discovery::feed_spec::FeedSpec;
use nostr_sdk::Keys;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn profile_feed_chases_its_creators_relay_lists() {
    let creator = Keys::generate().public_key();
    let mut runtime = runtime().await;
    let (bootstrap, mut probe) = test_bootstrap();
    runtime.bootstrap = bootstrap;

    runtime
        .open_feed(
            FeedSpec::Profile(vec![creator]),
            None,
            runtime.session_generation(),
        )
        .await
        .expect("profile feed opens");

    let chase = timeout(Duration::from_secs(5), probe.started.recv())
        .await
        .expect("relay-list chase should start")
        .expect("bootstrap recorder should stay open");
    let authors = &chase.plan.queries[0].filter.authors;
    assert!(authors.as_ref().is_some_and(|keys| keys.contains(&creator)));
}
