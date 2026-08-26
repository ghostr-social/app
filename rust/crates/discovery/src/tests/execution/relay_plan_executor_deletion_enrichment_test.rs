use crate::plan_executor::{PlanExecutor as _, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::deletion_enrichment_support::{executor, DeletionIo};
use crate::tests::support::filter_json;
use nostr_sdk::{EventBuilder, JsonUtil as _, Keys, Kind, Tag, Timestamp};

#[tokio::test]
async fn feed_fetches_original_authors_deletion_for_a_repost() {
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
        .tags([tag(&["e", &original.id.to_hex()])])
        .custom_created_at(Timestamp::from(30))
        .sign_with_keys(&creator)
        .expect("deletion");
    let io = DeletionIo::new(wrapper.clone(), deletion.clone());

    let events = executor(std::sync::Arc::clone(&io))
        .execute(retrieval(reposter.public_key()))
        .await
        .expect("feed retrieval");

    assert!(events.iter().any(|event| event.id == deletion.id));
    let filters = io.filters.lock().expect("filters");
    assert!(filters.iter().map(filter_json).any(|filter| {
        filter["kinds"] == serde_json::json!([5])
            && filter["authors"].as_array().is_some_and(|values| {
                values.contains(&serde_json::json!(creator.public_key().to_hex()))
            })
            && filter["#e"]
                .as_array()
                .is_some_and(|values| values.contains(&serde_json::json!(original.id.to_hex())))
    }));
}

fn retrieval(reposter: nostr_sdk::PublicKey) -> PlannedRetrieval {
    let request = DiscoveryRequest {
        authors: vec![reposter],
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
