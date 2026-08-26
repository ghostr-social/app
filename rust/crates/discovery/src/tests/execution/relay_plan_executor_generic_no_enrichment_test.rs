use crate::plan_executor::{PlanExecutor as _, PlannedRetrieval};
use crate::query::events::plan_event_queries;
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::profile_enrichment_support::{executor, has_kind, ProfileIo};
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
        context: FeedContext::for_session(
            "query",
            crate::session_generation::SessionGeneration::initial(),
        ),
        priority: RetrievalPriority::Enrichment,
        plan: plan_event_queries(vec![Filter::new().kind(Kind::Reaction)]),
        deferred_reposts: Vec::new(),
    };

    let events = executor(std::sync::Arc::clone(&io))
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
