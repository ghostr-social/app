//! Async observations shared by scheduler behavior tests.

use crate::plan_executor::PlannedRetrieval;
use crate::retrieval_types::RetrievalOutcome;
use core::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

pub(super) async fn next_started(
    started: &mut mpsc::UnboundedReceiver<PlannedRetrieval>,
) -> PlannedRetrieval {
    timeout(Duration::from_secs(5), started.recv())
        .await
        .expect("a retrieval should start")
        .expect("scheduler should stay alive")
}

pub(super) async fn no_start(started: &mut mpsc::UnboundedReceiver<PlannedRetrieval>) {
    let result = timeout(Duration::from_millis(50), started.recv()).await;
    assert!(result.is_err(), "no further retrieval may start");
}

pub(super) async fn next_outcome(
    outcomes: &mut mpsc::UnboundedReceiver<RetrievalOutcome>,
) -> RetrievalOutcome {
    timeout(Duration::from_secs(5), outcomes.recv())
        .await
        .expect("an outcome should arrive")
        .expect("scheduler should stay alive")
}
