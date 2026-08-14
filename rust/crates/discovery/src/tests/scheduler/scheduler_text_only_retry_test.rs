use crate::retrieval_types::{PlanFailure, RetrievalOutcome};
use crate::tests::scheduler_support::{context, next_outcome, next_started, note_at, request};
use crate::tests::scripted_scheduler_support::scripted_scheduler_results;
use nostr_sdk::{EventBuilder, Keys};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn text_only_page_does_not_suppress_a_later_failure_retry() {
    let text = EventBuilder::text_note("ordinary text")
        .sign_with_keys(&Keys::generate())
        .expect("text note");
    let mut harness = scripted_scheduler_results(vec![
        Ok(vec![text]),
        Err(PlanFailure::new("offline")),
        Ok(vec![note_at(40)]),
    ]);
    harness.handle.open_feed(context("profile"), request());

    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { result: Err(_), .. }
    ));

    tokio::time::advance(Duration::from_secs(1)).await;
    next_started(&mut harness.started).await;
}
