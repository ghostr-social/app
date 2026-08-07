use crate::tests::scheduler_support::{context, next_outcome, next_started, no_start, note_at, request};
use crate::tests::scripted_scheduler_support::scripted_scheduler_results;
use crate::retrieval_types::{PlanFailure, RetrievalOutcome};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn playable_head_work_cancels_a_pending_retry() {
    let event = note_at(40);
    let mut harness = scripted_scheduler_results(vec![
        Err(PlanFailure::new("offline")),
        Ok(vec![event.clone()]),
    ]);
    let feed = context("main");
    harness.handle.open_feed(feed.clone(), request());
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;

    harness.handle.background(feed, request());
    next_started(&mut harness.started).await;
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Started { .. }
    ));
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { result: Ok(events), .. } if events == vec![event]
    ));

    tokio::time::advance(Duration::from_millis(750)).await;
    no_start(&mut harness.started).await;
}

#[tokio::test(start_paused = true)]
async fn queued_retry_is_ignored_after_playable_head_work() {
    let event = note_at(40);
    let mut harness =
        scripted_scheduler_results(vec![Err(PlanFailure::new("offline")), Ok(vec![event])]);
    let feed = context("main");
    harness.handle.open_feed(feed.clone(), request());
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;

    harness.handle.background(feed.clone(), request());
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;
    next_outcome(&mut harness.outcomes).await;

    harness.handle.inject_retry(feed, 1);
    tokio::task::yield_now().await;

    no_start(&mut harness.started).await;
}
