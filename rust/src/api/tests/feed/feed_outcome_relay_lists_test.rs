//! Relay lists are filed on the way through the outcome pump, whichever
//! retrieval carried them: the viewer's kind-3 becomes the main feed's
//! routing set, the follows' kind-10002 becomes the relays it routes to,
//! and follows nobody asked about yet are chased.

use crate::api::feed::state::FeedState;
use crate::api::runtime::discovery::{lock, pump_outcomes, OutcomeSinks, SharedFeedState};
use crate::api::tests::feed_fixtures::{relay_list_event, signed_event, SignedEventFixture};
use crate::api::tests::outbox_runtime_support::{test_bootstrap, BOOTSTRAP_RELAY};
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::outbox::bootstrap::OUTBOX_CONTEXT;
use crate::discovery::retrieval_types::{FeedContext, RetrievalOutcome, RetrievalPurpose};
use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::{Event, Keys, Kind, PublicKey};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

fn contact_list(viewer: &Keys, follow: &PublicKey) -> Event {
    let tags = vec![vec!["p".to_owned(), follow.to_hex()]];
    signed_event(SignedEventFixture {
        keys: viewer,
        kind: Kind::ContactList,
        content: "",
        tags,
        created_at: 20,
    })
}

fn opened_state(viewer: &Keys) -> SharedFeedState {
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });
    state
}

#[tokio::test]
async fn a_landed_follow_list_routes_the_feed_to_the_follows_relays() {
    let viewer = Keys::generate();
    let follow = Keys::generate();
    let state = opened_state(&viewer);
    let (bootstrap, mut probe) = test_bootstrap();
    let (sender, outcomes) = mpsc::unbounded_channel();
    tokio::spawn(pump_outcomes(
        OutcomeSinks {
            state,
            bootstrap,
            candidates: None,
        },
        outcomes,
    ));

    sender
        .send(RetrievalOutcome::Completed {
            context: FeedContext::for_session(OUTBOX_CONTEXT, SessionGeneration::initial()),
            result: Ok(vec![
                contact_list(&viewer, &follow.public_key()),
                relay_list_event(&follow, &["wss://follow.write"], 10),
            ]),
            cursor: None,
            complete: true,
            purpose: RetrievalPurpose::Head,
        })
        .expect("the pump should be listening");

    let chased = timeout(Duration::from_secs(5), probe.started.recv())
        .await
        .expect("the follows' relay lists should be chased")
        .expect("the recorder should stay alive");
    assert_eq!(chased.plan.queries.len(), 1);
    let relays = probe.directory.read().await.discovery_relays(12);
    assert_eq!(
        relays,
        vec![BOOTSTRAP_RELAY.to_owned(), "wss://follow.write".to_owned()]
    );
}
