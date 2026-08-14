use crate::execution::collector::collect_best_effort_events;
use crate::execution::fetch::FetchedEvents;
use crate::query::search::QueryRole;
use crate::retrieval_types::PlanFailure;
use nostr_sdk::{EventBuilder, Keys};

#[tokio::test]
async fn best_effort_enrichment_keeps_successes_when_one_query_fails() {
    let event = EventBuilder::text_note("profile")
        .sign_with_keys(&Keys::generate())
        .expect("event");
    let success = tokio::spawn({
        let event = event.clone();
        async move { Ok(FetchedEvents::fresh(vec![event])) }
    });
    let failure = tokio::spawn(async { Err(PlanFailure::new("offline relay")) });

    let events = collect_best_effort_events(vec![
        (QueryRole::Additive, success),
        (QueryRole::Additive, failure),
    ])
    .await;

    assert_eq!(events, vec![event]);
}
