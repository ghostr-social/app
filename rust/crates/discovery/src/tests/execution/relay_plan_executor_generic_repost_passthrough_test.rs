use crate::plan_executor::{PlanExecutor as _, PlannedRetrieval};
use crate::query::events::plan_event_queries;
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::profile_enrichment_support::{executor, ProfileIo};
use nostr_sdk::{EventBuilder, Filter, Keys, Kind};

#[tokio::test]
async fn generic_enrichment_returns_raw_repost_wrappers_unchanged() {
    let wrapper = EventBuilder::new(Kind::Custom(16), "")
        .sign_with_keys(&Keys::generate())
        .expect("wrapper");
    let io = ProfileIo::new(Kind::Custom(16), wrapper.clone(), wrapper.clone());
    let retrieval = PlannedRetrieval {
        context: FeedContext::for_session(
            "query",
            crate::session_generation::SessionGeneration::initial(),
        ),
        priority: RetrievalPriority::Enrichment,
        plan: plan_event_queries(vec![Filter::new().kind(Kind::Custom(16))]),
        deferred_reposts: Vec::new(),
    };

    let events = executor(io)
        .execute(retrieval)
        .await
        .expect("generic repost read");

    assert_eq!(
        events.iter().map(|event| event.id).collect::<Vec<_>>(),
        [wrapper.id]
    );
}
