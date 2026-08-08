use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::events::plan_event_queries;
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::profile_enrichment_support::{executor, has_kind, ProfileIo};
use nostr_sdk::{EventBuilder, Filter, Keys, Kind};

#[tokio::test]
async fn outbox_results_do_not_trigger_creator_metadata() {
    let author = Keys::generate();
    let relay_list = EventBuilder::new(Kind::RelayList, "")
        .sign_with_keys(&author)
        .expect("relay list");
    let profile = EventBuilder::new(Kind::Metadata, "{}")
        .sign_with_keys(&author)
        .expect("profile");
    let io = ProfileIo::new(Kind::RelayList, relay_list.clone(), profile);
    let retrieval = PlannedRetrieval {
        context: FeedContext::for_session(
            "outbox",
            crate::session_generation::SessionGeneration::initial(),
        ),
        priority: RetrievalPriority::Background,
        plan: plan_event_queries(vec![Filter::new().kind(Kind::RelayList)]),
    };

    let events = executor(io.clone())
        .execute(retrieval)
        .await
        .expect("outbox read");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, relay_list.id);
    assert!(!io
        .filters
        .lock()
        .expect("filters")
        .iter()
        .any(|filter| has_kind(filter, Kind::Metadata)));
}
