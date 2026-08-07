use super::profile_enrichment_support::{executor, has_kind, ProfileIo};
use crate::event_queries::plan_event_queries;
use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::retrieval_queue::{FeedContext, RetrievalPriority};
use nostr_sdk::{EventBuilder, Filter, Keys, Kind};

#[tokio::test]
async fn generic_reads_do_not_wait_for_creator_metadata() {
    let author = Keys::generate();
    let reaction = EventBuilder::new(Kind::Reaction, "+")
        .sign_with_keys(&author)
        .expect("reaction");
    let profile = EventBuilder::new(Kind::Metadata, r#"{"name":"Vera"}"#)
        .sign_with_keys(&author)
        .expect("profile");
    let io = ProfileIo::new(Kind::Reaction, reaction.clone(), profile);
    let retrieval = PlannedRetrieval {
        context: FeedContext::new("query"),
        priority: RetrievalPriority::Enrichment,
        plan: plan_event_queries(vec![Filter::new().kind(Kind::Reaction)]),
    };

    let events = executor(io.clone())
        .execute(retrieval)
        .await
        .expect("generic read");

    assert_eq!(
        events.iter().map(|event| event.id).collect::<Vec<_>>(),
        [reaction.id]
    );
    assert!(!io
        .filters
        .lock()
        .expect("filters")
        .iter()
        .any(|filter| has_kind(filter, Kind::Metadata)));
}
