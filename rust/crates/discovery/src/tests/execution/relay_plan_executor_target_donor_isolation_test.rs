use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::{DiscoveryRequest, RepostAdmission};
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::selective_deletion_support::{selective_deletion_executor, SelectiveDeletionIo};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};

const BAD_RELAY: &str = "wss://bad-donor.example";
const GOOD_RELAY: &str = "wss://good-donor.example";

#[tokio::test]
async fn failed_deletion_drops_only_its_enrichment_only_donor() {
    let reposter = Keys::generate();
    let bad = protected_repost(&reposter, BAD_RELAY);
    let good = protected_repost(&reposter, GOOD_RELAY);
    let base = video(&reposter);
    let events = vec![
        bad.0.clone(),
        good.0.clone(),
        bad.1.clone(),
        good.1.clone(),
        base.clone(),
    ];
    let io = SelectiveDeletionIo::new(events, BAD_RELAY);
    let (progress, _) = tokio::sync::mpsc::channel(1);

    let page = selective_deletion_executor(io)
        .execute_page_with_progress(retrieval(reposter.public_key()), progress)
        .await
        .expect("content page");

    assert!(has(&page.events, good.0.id));
    assert!(has(&page.events, good.1.id));
    assert!(has(&page.events, base.id));
    assert!(!has(&page.events, bad.0.id));
    assert!(!has(&page.events, bad.1.id));
    assert_eq!(page.repost_retry.deferred[0].id, bad.0.id);
}

fn protected_repost(reposter: &Keys, relay: &str) -> (Event, Event) {
    let creator = Keys::generate();
    let original = video(&creator);
    let wrapper = EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&["e", &original.id.to_hex(), relay]),
            tag(&["p", &creator.public_key().to_hex()]),
            tag(&["k", "21"]),
        ])
        .sign_with_keys(reposter)
        .expect("wrapper");
    (wrapper, original)
}

fn video(keys: &Keys) -> Event {
    EventBuilder::new(Kind::Custom(21), "https://cdn.example/video.mp4")
        .sign_with_keys(keys)
        .expect("video")
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

fn has(events: &[Event], id: nostr_sdk::EventId) -> bool {
    events.iter().any(|event| event.id == id)
}
