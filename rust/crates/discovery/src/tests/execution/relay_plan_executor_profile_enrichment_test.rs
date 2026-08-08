use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::DiscoveryRequest;
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::profile_enrichment_support::{executor, has_kind, ProfileIo};
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
        context: FeedContext::for_session(
            "feed",
            crate::session_generation::SessionGeneration::initial(),
        ),
        priority: RetrievalPriority::Interactive,
        plan: plan_discovery(&DiscoveryRequest::default()),
    }
}
