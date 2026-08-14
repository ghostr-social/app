use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::repost_target_executor_support::target_executor;
use crate::tests::repost_target_support::{RepostTargetIo, TARGET_RELAY};
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

#[tokio::test]
async fn fetched_invalid_specific_replaceable_target_is_not_retried() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(34235), "https://cdn.example/video.mp4")
        .tags([tag(&["d", "clip"])])
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&["e", &original.id.to_hex(), TARGET_RELAY]),
            tag(&["p", &creator.public_key().to_hex()]),
        ])
        .sign_with_keys(&reposter)
        .expect("wrapper");
    let executor = target_executor(RepostTargetIo::new(wrapper.clone(), original));
    let request = DiscoveryRequest {
        authors: vec![reposter.public_key()],
        reposts: RepostAdmission::Included,
        ..DiscoveryRequest::default()
    };
    let (progress, _) = tokio::sync::mpsc::channel(1);

    let page = executor
        .execute_page_with_progress(retrieval(request), progress)
        .await
        .expect("content page");

    assert!(!page.events.iter().any(|event| event.id == wrapper.id));
    assert!(page.repost_retry.deferred.is_empty());
}

fn retrieval(request: DiscoveryRequest) -> PlannedRetrieval {
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
