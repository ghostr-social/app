use crate::retrieval_types::{PlanFailure, RetrievalOutcome, RetrievalPurpose};
use crate::tests::scheduler_support::{
    context, next_outcome, next_started, no_start, note_at, request,
};
use crate::tests::scripted_scheduler_support::scripted_scheduler_results;
use nostr_sdk::Timestamp;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn an_older_page_failure_never_turns_into_a_head_retry() {
    let mut harness = scripted_scheduler_results(vec![
        Ok(vec![note_at(40)]),
        Err(PlanFailure::new("older offline")),
    ]);
    let feed = context("main");
    harness.handle.open_feed(feed.clone(), request());
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;

    harness.handle.load_more(feed, Some(Timestamp::from(39)));
    next_started(&mut harness.started).await;
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed {
            result: Err(_),
            purpose: RetrievalPurpose::Older,
            ..
        }
    ));

    tokio::time::advance(Duration::from_millis(750)).await;
    no_start(&mut harness.started).await;
}
