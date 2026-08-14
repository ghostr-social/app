use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::repost_target_executor_support::target_executor;
use crate::tests::repost_target_support::{RepostTargetIo, TARGET_RELAY};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

#[tokio::test]
async fn failed_target_lookup_is_deferred_across_the_wire_cursor() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/v.mp4")
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
    let io = RepostTargetIo::failing(wrapper.clone(), original);

    let executor = target_executor(io);
    let first = page(
        &executor,
        retrieval(reposter.public_key(), None, Vec::new()),
    )
    .await;
    assert!(!first.events.iter().any(|event| event.id == wrapper.id));
    assert_eq!(first.repost_retry.deferred[0].id, wrapper.id);

    let retried = page(
        &executor,
        retrieval(
            reposter.public_key(),
            Some(Timestamp::from(0)),
            first.repost_retry.deferred,
        ),
    )
    .await;

    assert!(retried.events.iter().any(|event| event.id == wrapper.id));
    assert!(retried.repost_retry.deferred.is_empty());
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
