use super::profile_enrichment_support::{executor, has_kind, ProfileIo};
use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::search_queries::plan_discovery;
use crate::video_filters::DiscoveryRequest;
use nostr_sdk::{EventBuilder, Keys, Kind, Timestamp};

#[tokio::test]
async fn feed_execution_loads_metadata_for_the_returned_creator() {
    let creator = Keys::generate();
    let profile = EventBuilder::new(Kind::Metadata, r#"{"name":"Vera"}"#)
        .custom_created_at(Timestamp::from(5))
        .sign_with_keys(&creator)
        .expect("profile");
    let video = EventBuilder::text_note("https://cdn.example/clip.mp4")
        .custom_created_at(Timestamp::from(40))
        .sign_with_keys(&creator)
        .expect("video");
    let io = ProfileIo::new(Kind::TextNote, video, profile.clone());

    let events = executor(io.clone())
        .execute(retrieval())
        .await
        .expect("feed retrieval");

    assert!(events.iter().any(|event| event.id == profile.id));
    let filters = io.filters.lock().expect("filters");
    let metadata = filters
        .iter()
        .find(|filter| has_kind(filter, Kind::Metadata))
        .expect("a metadata query follows the video query");
    assert!(metadata
        .authors
        .as_ref()
        .is_some_and(|authors| authors.contains(&creator.public_key())));
}

fn retrieval() -> PlannedRetrieval {
    PlannedRetrieval {
        context: FeedContext::new("feed"),
        priority: RetrievalPriority::Interactive,
        plan: plan_discovery(&DiscoveryRequest::default()),
    }
}
