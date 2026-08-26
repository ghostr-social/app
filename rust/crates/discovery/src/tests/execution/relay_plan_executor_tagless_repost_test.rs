use crate::plan_executor::{PlanExecutor as _, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::repost_target_executor_support::target_executor;
use crate::tests::repost_target_support::RepostTargetIo;
use nostr_sdk::{EventBuilder, JsonUtil as _, Keys, Kind};

#[tokio::test]
async fn embedded_kind_sixteen_without_reference_tags_survives_execution() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/v.mp4")
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Custom(16), original.as_json())
        .sign_with_keys(&reposter)
        .expect("wrapper");
    let io = RepostTargetIo::new(wrapper.clone(), original);

    let events = target_executor(io)
        .execute(retrieval(reposter.public_key()))
        .await
        .expect("content page");

    assert!(events.iter().any(|event| event.id == wrapper.id));
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
