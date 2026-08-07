//! A cold canonical feed recovers without user input after its first
//! relay attempt fails.

use super::scheduler_support::{context, next_outcome, next_started, no_start, note_at, request};
use super::scripted_scheduler_support::scripted_scheduler_results;
use crate::retrieval_types::{PlanFailure, RetrievalOutcome};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn failed_first_page_retries_and_lands_content() {
    let event = note_at(40);
    let mut harness = scripted_scheduler_results(vec![
        Err(PlanFailure::new("offline")),
        Ok(vec![event.clone()]),
    ]);
    harness.handle.open_feed(context("main"), request());
    next_started(&mut harness.started).await;
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { result: Err(_), .. }
    ));

    tokio::time::advance(Duration::from_millis(500)).await;
    next_started(&mut harness.started).await;
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Started { .. }
    ));
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { result: Ok(events), .. } if events == vec![event]
    ));
}

#[tokio::test(start_paused = true)]
async fn scheduled_retry_never_overlaps_active_context_work() {
    let mut harness = scripted_scheduler_results(vec![Err(PlanFailure::new("offline"))]);
    let feed = context("main");
    harness.handle.open_feed(feed.clone(), request());
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;

    harness.handle.background(feed, request());
    next_started(&mut harness.started).await;
    tokio::time::advance(Duration::from_millis(500)).await;

    no_start(&mut harness.started).await;
}

#[tokio::test(start_paused = true)]
async fn closing_feed_cancels_its_scheduled_retry() {
    let mut harness = scripted_scheduler_results(vec![Err(PlanFailure::new("offline"))]);
    let feed = context("main");
    harness.handle.open_feed(feed.clone(), request());
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;

    harness.handle.close_feed(feed);
    tokio::time::advance(Duration::from_millis(500)).await;

    no_start(&mut harness.started).await;
}
