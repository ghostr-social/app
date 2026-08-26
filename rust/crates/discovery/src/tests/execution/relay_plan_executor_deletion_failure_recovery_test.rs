use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::deletion_failure_support::{deletion_failure_executor, DeletionFailureIo};
use nostr_sdk::{Event, EventBuilder, JsonUtil as _, Keys, Kind, Tag, Timestamp};

#[tokio::test]
async fn failed_deletion_lookup_keeps_safe_rows_and_retries_the_wrapper() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/o.mp4")
        .custom_created_at(Timestamp::from(10))
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Custom(16), original.as_json())
        .tags([tag(&["e", &original.id.to_hex()])])
        .custom_created_at(Timestamp::from(20))
        .sign_with_keys(&reposter)
        .expect("wrapper");
    let safe = EventBuilder::new(Kind::Custom(21), "https://cdn.example/s.mp4")
        .custom_created_at(Timestamp::from(15))
        .sign_with_keys(&reposter)
        .expect("safe video");
    let executor = deletion_failure_executor(DeletionFailureIo::new(wrapper.clone(), safe.clone()));
    let first_retrieval = retrieval(reposter.public_key(), None, Vec::new());

    let first = page(&executor, first_retrieval).await;
    assert!(first.repost_retry.is_pending());
    assert!(first.events.iter().any(|event| event.id == safe.id));
    assert!(!first.events.iter().any(|event| event.id == wrapper.id));

    let deferred = first.repost_retry.deferred.clone();
    let older = retrieval(reposter.public_key(), Some(Timestamp::from(9)), deferred);
    let retried = page(&executor, older).await;
    assert!(!retried.repost_retry.is_pending());
    assert!(retried.events.iter().any(|event| event.id == wrapper.id));
}

async fn page(
    executor: &impl PlanExecutor,
    retrieval: PlannedRetrieval,
) -> crate::plan_executor::PlanPage {
    let (progress, _) = tokio::sync::mpsc::channel(1);
    executor
        .execute_page_with_progress(retrieval, progress)
        .await
        .expect("content page")
}

fn retrieval(
    reposter: nostr_sdk::PublicKey,
    older_than: Option<Timestamp>,
    deferred_reposts: Vec<Event>,
) -> PlannedRetrieval {
    let request = DiscoveryRequest {
        authors: vec![reposter],
        reposts: RepostAdmission::Included,
        older_than,
        ..DiscoveryRequest::default()
    };
    PlannedRetrieval {
        context: FeedContext::for_session(
            "following",
            crate::session_generation::SessionGeneration::initial(),
        ),
        priority: RetrievalPriority::Interactive,
        plan: plan_discovery(&request),
        deferred_reposts,
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
