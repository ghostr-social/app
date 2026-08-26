use crate::plan_executor::{PlanExecutor as _, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::repost_target_executor_support::target_executor;
use crate::tests::repost_target_support::{RepostTargetIo, TARGET_RELAY};
use core::sync::atomic::Ordering;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

#[tokio::test]
async fn hinted_original_deletion_is_retrieved_with_the_protected_repost() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/v.mp4")
        .tags([tag(&["-"])])
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
    let deletion = EventBuilder::new(Kind::EventDeletion, "deleted")
        .tags([tag(&["e", &original.id.to_hex()])])
        .sign_with_keys(&creator)
        .expect("deletion");
    let io = RepostTargetIo::with_deletion(wrapper, original, deletion.clone());

    let events = target_executor(std::sync::Arc::clone(&io))
        .execute(retrieval(reposter.public_key()))
        .await
        .expect("feed retrieval");

    assert!(events.iter().any(|event| event.id == deletion.id));
    assert!(io.used_deletion_hint.load(Ordering::Relaxed));
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
