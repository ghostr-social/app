//! Reset invalidates bootstrap contexts and releases every author claim.

use crate::discovery::discovery_scheduler::RetrievalOutcome;
use crate::discovery::outbox_bootstrap::OutboxBootstrap;
use crate::discovery::session_generation::SessionGeneration;
use crate::discovery::tests::outbox_support::{empty_directory, recording_executor};
use nostr_sdk::Keys;
use tokio::sync::mpsc;

#[tokio::test]
async fn reset_reissues_claims_under_the_new_session() {
    let viewer = Keys::generate().public_key();
    let (executor, mut retrievals) = recording_executor();
    let (outcomes, _receiver) = mpsc::unbounded_channel::<RetrievalOutcome>();
    let bootstrap = OutboxBootstrap::new(executor, empty_directory(), outcomes);
    bootstrap.viewer(viewer);
    let stale = retrievals.recv().await.expect("old retrieval");
    let fresh = SessionGeneration::initial().next();

    bootstrap.reset_session(fresh);
    bootstrap.viewer(viewer);
    let renewed = retrievals.recv().await.expect("renewed retrieval");

    assert_ne!(stale.context.session(), renewed.context.session());
    assert_eq!(renewed.context.session(), fresh);
}
