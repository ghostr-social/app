use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::repost_target_executor_support::target_executor;
use crate::tests::repost_target_support::{RepostTargetIo, TARGET_RELAY};
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use std::sync::atomic::Ordering;

#[tokio::test]
async fn unverified_exact_target_remains_retryable() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let mut original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/video.mp4")
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
    original.content.push_str("?tampered");
    let io = RepostTargetIo::new(wrapper.clone(), original);
    let executor = target_executor(io.clone());
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
    assert_eq!(page.repost_retry.deferred[0].id, wrapper.id);
    assert!(io.used_hint.load(Ordering::Relaxed));
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
