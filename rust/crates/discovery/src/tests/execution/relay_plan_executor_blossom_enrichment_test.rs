use crate::content::candidates::CandidateRegistry;
use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::query::search::plan_discovery;
use crate::query::video_filters::DiscoveryRequest;
use crate::retrieval_types::{FeedContext, RetrievalPriority};
use crate::tests::blossom_enrichment_support::{executor, has_kind, BlossomIo};
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

#[tokio::test]
async fn feed_execution_loads_the_media_authors_blossom_server_list() {
    let author = Keys::generate();
    let digest = "a".repeat(64);
    let video = plain_x_video(&author, &digest);
    let servers = EventBuilder::new(Kind::Custom(10063), "")
        .tags([Tag::parse(["server", "https://blossom.example"]).expect("server")])
        .sign_with_keys(&author)
        .expect("server list");
    let io = BlossomIo::new(video, servers.clone());

    let events = executor(io.clone())
        .execute(retrieval())
        .await
        .expect("feed retrieval");

    assert!(events.iter().any(|event| event.id == servers.id));
    let candidate = CandidateRegistry::new()
        .inspect_all(&events)
        .admitted
        .into_iter()
        .next()
        .expect("candidate")
        .post;
    let mirror = format!("https://blossom.example/{digest}");
    assert!(candidate.meta.urls.contains(&mirror));
    assert!(candidate.metadata_evidence[0].urls.contains(&mirror));
    let filters = io.filters.lock().expect("filters");
    let blossom = filters
        .iter()
        .find(|filter| has_kind(filter, Kind::Custom(10063)))
        .expect("kind-10063 enrichment query");
    assert!(blossom
        .authors
        .as_ref()
        .is_some_and(|authors| authors.contains(&author.public_key())));
}

fn plain_x_video(author: &Keys, digest: &str) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(22), "clip")
        .tags([Tag::parse([
            "imeta".to_owned(),
            "url https://cdn.example/plain.mp4".to_owned(),
            "m video/mp4".to_owned(),
            format!("x {digest}"),
        ])
        .expect("imeta")])
        .sign_with_keys(author)
        .expect("video")
}

fn retrieval() -> PlannedRetrieval {
    PlannedRetrieval {
        context: FeedContext::for_session(
            "feed",
            crate::session_generation::SessionGeneration::initial(),
        ),
        priority: RetrievalPriority::Interactive,
        plan: plan_discovery(&DiscoveryRequest::default()),
        deferred_reposts: Vec::new(),
    }
}
