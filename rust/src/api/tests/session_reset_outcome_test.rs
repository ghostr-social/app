//! A late old-feed outcome cannot restore account routing after reset.

use crate::api::feed_runtime::{lock, pump_outcomes, OutcomeSinks, SharedFeedState};
use crate::api::feed_state::FeedState;
use crate::api::tests::feed_fixtures::{relay_list_event, signed_event, SignedEventFixture};
use crate::api::tests::outbox_runtime_support::{test_bootstrap, BOOTSTRAP_RELAY};
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::retrieval_types::{RetrievalOutcome, RetrievalPurpose};
use nostr_sdk::{Event, Keys, Kind, PublicKey};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

fn contact_list(viewer: &Keys, follow: PublicKey) -> Event {
    let tags = vec![vec!["p".to_owned(), follow.to_hex()]];
    signed_event(SignedEventFixture {
        keys: viewer,
        kind: Kind::ContactList,
        content: "",
        tags,
        created_at: 20,
    })
}

#[tokio::test]
async fn old_outcome_is_ignored_after_a_new_session_opens() {
    let viewer = Keys::generate();
    let follow = Keys::generate();
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let (_, stale) = lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });
    let stale_context = stale.expect("old feed").context;
    let generation = lock(&state).reset_session();
    lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });
    let (bootstrap, probe) = test_bootstrap();
    bootstrap.reset_session(generation);
    probe.directory.write().await.reset_session(generation);
    let (sender, outcomes) = mpsc::unbounded_channel();
    let pump = tokio::spawn(pump_outcomes(
        OutcomeSinks {
            state: state.clone(),
            bootstrap,
            candidates: None,
        },
        outcomes,
    ));

    sender
        .send(RetrievalOutcome::Completed {
            context: stale_context,
            result: Ok(vec![
                contact_list(&viewer, follow.public_key()),
                relay_list_event(&follow, &["wss://old.example"], 10),
            ]),
            purpose: RetrievalPurpose::Head,
        })
        .expect("pump");
    drop(sender);
    pump.await.expect("pump completion");

    let (_, fresh) = lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });
    assert!(fresh
        .expect("fresh feed")
        .request
        .routing_authors
        .is_empty());
    assert_eq!(
        probe.directory.read().await.discovery_relays(12),
        vec![BOOTSTRAP_RELAY.to_owned()]
    );
}
