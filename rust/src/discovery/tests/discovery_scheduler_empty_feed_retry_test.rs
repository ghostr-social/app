//! A cold canonical feed keeps trying when relays settle its first
//! page without any playable content.

use super::scheduler_support::{context, next_outcome, next_started, note_at, request};
use super::scripted_scheduler_support::scripted_scheduler;
use crate::discovery::discovery_scheduler::RetrievalOutcome;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn empty_first_page_retries_and_lands_content() {
    let event = note_at(40);
    let mut harness = scripted_scheduler(vec![Vec::new(), vec![event.clone()]]);
    harness.handle.open_feed(context("main"), request());
    next_started(&mut harness.started).await;
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { result: Ok(events), .. } if events.is_empty()
    ));

    tokio::time::advance(Duration::from_millis(500)).await;
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { result: Ok(events), .. } if events == vec![event]
    ));
}
