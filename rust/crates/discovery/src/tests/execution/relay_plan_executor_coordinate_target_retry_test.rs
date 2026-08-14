use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::repost_target_executor_support::target_executor;
use crate::tests::repost_target_support::{RepostTargetIo, TARGET_RELAY};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

#[tokio::test]
async fn empty_coordinate_answer_is_deferred_until_its_current_target_arrives() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = addressable_video(&creator);
    let coordinate = format!("34235:{}:clip", creator.public_key());
    let wrapper = EventBuilder::new(Kind::Custom(16), "")
        .tags([tag(&["a", &coordinate, TARGET_RELAY])])
        .sign_with_keys(&reposter)
        .expect("wrapper");
    let executor = target_executor(RepostTargetIo::empty_once(
        wrapper.clone(),
        original.clone(),
    ));

    let first = page(&executor, retrieval(&reposter, None, Vec::new())).await;
    assert!(!first.events.iter().any(|event| event.id == wrapper.id));
    assert_eq!(first.repost_retry.deferred[0].id, wrapper.id);

    let second = page(
        &executor,
        retrieval(
            &reposter,
            Some(Timestamp::from(0)),
            first.repost_retry.deferred,
        ),
    )
    .await;
    assert!(second.events.iter().any(|event| event.id == wrapper.id));
    assert!(second.events.iter().any(|event| event.id == original.id));
    assert!(second.repost_retry.deferred.is_empty());
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
    reposter: &Keys,
    older_than: Option<Timestamp>,
    deferred_reposts: Vec<Event>,
) -> PlannedRetrieval {
    let request = DiscoveryRequest {
        authors: vec![reposter.public_key()],
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

fn addressable_video(keys: &Keys) -> Event {
    EventBuilder::new(Kind::Custom(34235), "https://cdn.example/video.mp4")
        .tags([tag(&["d", "clip"])])
        .sign_with_keys(keys)
        .expect("original")
}

fn tag(values: &[&str]) -> Tag {
    Tag::parse(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("tag")
}
