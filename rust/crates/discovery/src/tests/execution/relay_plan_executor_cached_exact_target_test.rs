use crate::plan_executor::{PlanExecutor as _, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::session_generation::SessionGeneration;
use crate::tests::repost_target_executor_support::target_executor;
use crate::tests::repost_target_support::{RepostTargetIo, TARGET_RELAY};
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

#[tokio::test]
async fn verified_exact_target_settles_when_its_redundant_lookup_fails() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/video.mp4")
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&["e", &original.id.to_hex(), TARGET_RELAY]),
            tag(&["p", &creator.public_key().to_hex()]),
            tag(&["k", "21"]),
        ])
        .sign_with_keys(&reposter)
        .expect("wrapper");
    let executor = target_executor(RepostTargetIo::failing(wrapper.clone(), original.clone()));
    executor
        .cache()
        .remember_for(SessionGeneration::initial(), &[original])
        .await;
    let (progress, _) = tokio::sync::mpsc::channel(1);

    let page = executor
        .execute_page_with_progress(retrieval(&reposter, &creator), progress)
        .await
        .expect("content page");

    assert!(page.events.iter().any(|event| event.id == wrapper.id));
    assert!(page.repost_retry.deferred.is_empty());
}

fn retrieval(reposter: &Keys, creator: &Keys) -> PlannedRetrieval {
    let request = DiscoveryRequest {
        authors: vec![reposter.public_key(), creator.public_key()],
        reposts: RepostAdmission::Included,
        ..DiscoveryRequest::default()
    };
    PlannedRetrieval {
        context: FeedContext::for_session("following", SessionGeneration::initial()),
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
