use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::deletion_enrichment_support::{executor, DeletionIo};
use nostr_sdk::{EventBuilder, JsonUtil, Keys, Kind, Tag, Timestamp};

#[tokio::test]
async fn historical_feed_fetches_a_newer_wrapper_deletion() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/v.mp4")
        .custom_created_at(Timestamp::from(10))
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Custom(16), original.as_json())
        .tags([tag(&["e", &original.id.to_hex()])])
        .custom_created_at(Timestamp::from(20))
        .sign_with_keys(&reposter)
        .expect("wrapper");
    let deletion = EventBuilder::new(Kind::EventDeletion, "deleted")
        .tags([tag(&["e", &wrapper.id.to_hex()])])
        .custom_created_at(Timestamp::from(30))
        .sign_with_keys(&reposter)
        .expect("deletion");

    let events = executor(DeletionIo::new(wrapper, deletion.clone()))
        .execute(retrieval(reposter.public_key()))
        .await
        .expect("feed retrieval");

    assert!(events.iter().any(|event| event.id == deletion.id));
}

fn retrieval(reposter: nostr_sdk::PublicKey) -> PlannedRetrieval {
    let request = DiscoveryRequest {
        authors: vec![reposter],
        older_than: Some(Timestamp::from(25)),
        reposts: RepostAdmission::Included,
        ..DiscoveryRequest::default()
    };
    PlannedRetrieval {
        context: FeedContext::for_session(
            "following",
            crate::session_generation::SessionGeneration::initial(),
        ),
        priority: RetrievalPriority::Interactive,
        plan: plan_discovery(&request),
        deferred_reposts: Vec::new(),
    }
}

fn tag(values: &[&str]) -> Tag {
    Tag::parse(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("valid tag")
}
