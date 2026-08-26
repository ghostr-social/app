use crate::api::tests::outbox_runtime_support::test_bootstrap;
use crate::api::tests::runtime_fixture::runtime;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::plan_executor::PlannedRetrieval;
use core::time::Duration;
use nostr_sdk::{Keys, PublicKey};
use tokio::time::timeout;

#[tokio::test]
async fn following_feed_chases_viewer_and_follow_relay_lists() {
    let viewer = Keys::generate().public_key();
    let follow = Keys::generate().public_key();
    let mut runtime = runtime().await;
    let (bootstrap, mut probe) = test_bootstrap();
    runtime.bootstrap = bootstrap;
    runtime.reset_session(Some(viewer)).await;

    runtime
        .open_feed(
            FeedSpec::Following {
                viewer: Some(viewer),
                follows: vec![follow],
            },
            Some(viewer),
            runtime.session_generation(),
        )
        .await
        .expect("following feed opens");

    let first = started(&mut probe.started).await;
    let second = started(&mut probe.started).await;
    assert!(queries(&first, viewer) || queries(&second, viewer));
    assert!(queries(&first, follow) || queries(&second, follow));
}

async fn started(
    probe: &mut tokio::sync::mpsc::UnboundedReceiver<PlannedRetrieval>,
) -> PlannedRetrieval {
    timeout(Duration::from_secs(5), probe.recv())
        .await
        .expect("relay-list chase should start")
        .expect("bootstrap recorder should stay open")
}

fn queries(retrieval: &PlannedRetrieval, author: PublicKey) -> bool {
    retrieval.plan.queries.iter().any(|query| {
        query
            .filter
            .authors
            .as_ref()
            .is_some_and(|authors| authors.contains(&author))
    })
}
