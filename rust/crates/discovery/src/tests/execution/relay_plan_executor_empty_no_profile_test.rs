use crate::tests::profile_enrichment_support::{executor, has_kind, ProfileIo};
use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::query::search::plan_discovery;
use crate::query::video_filters::DiscoveryRequest;
use nostr_sdk::{EventBuilder, Keys, Kind};

#[tokio::test]
async fn empty_feed_result_skips_profile_enrichment() {
    let placeholder = EventBuilder::new(Kind::Metadata, "{}")
        .sign_with_keys(&Keys::generate())
        .expect("placeholder");
    let io = ProfileIo::empty(Kind::TextNote, placeholder);
    let retrieval = PlannedRetrieval {
        context: FeedContext::new("feed"),
        priority: RetrievalPriority::Interactive,
        plan: plan_discovery(&DiscoveryRequest::default()),
    };

    let events = executor(io.clone())
        .execute(retrieval)
        .await
        .expect("empty feed");

    assert!(events.is_empty());
    assert!(!io
        .filters
        .lock()
        .expect("filters")
        .iter()
        .any(|filter| has_kind(filter, Kind::Metadata)));
}
