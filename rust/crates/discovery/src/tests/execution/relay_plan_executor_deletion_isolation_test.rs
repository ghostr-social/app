use crate::plan_executor::{PlanExecutor as _, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::selective_deletion_support::{selective_deletion_executor, SelectiveDeletionIo};
use nostr_sdk::{Event, EventBuilder, JsonUtil as _, Keys, Kind, Tag};

const BAD_RELAY: &str = "wss://bad-deletions.example";
const GOOD_RELAY: &str = "wss://good-deletions.example";

#[tokio::test]
async fn one_failed_deletion_lookup_withholds_only_its_repost() {
    let reposter = Keys::generate();
    let bad = wrapper(&reposter, BAD_RELAY);
    let good = wrapper(&reposter, GOOD_RELAY);
    let io = SelectiveDeletionIo::new(vec![bad.clone(), good.clone()], BAD_RELAY);
    let (progress, _) = tokio::sync::mpsc::channel(1);

    let page = selective_deletion_executor(io)
        .execute_page_with_progress(retrieval(reposter.public_key()), progress)
        .await
        .expect("content page");

    assert!(page.events.iter().any(|event| event.id == good.id));
    assert!(!page.events.iter().any(|event| event.id == bad.id));
    assert_eq!(page.repost_retry.deferred[0].id, bad.id);
}

fn wrapper(reposter: &Keys, relay: &str) -> Event {
    let creator = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/v.mp4")
        .sign_with_keys(&creator)
        .expect("original");
    EventBuilder::new(Kind::Custom(16), original.as_json())
        .tags([tag(&["e", &original.id.to_hex(), relay])])
        .sign_with_keys(reposter)
        .expect("wrapper")
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
