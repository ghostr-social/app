use crate::api::feed::state::FeedState;
use crate::api::runtime::discovery::{lock, pump_outcomes, OutcomeSinks, SharedFeedState};
use crate::api::tests::outbox_runtime_support::test_bootstrap;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::retrieval_types::{RetrievalOutcome, RetrievalPurpose};
use core::time::Duration;
use ghostr_delivery::delivery_events::{command_channel, CommandReceiver, DeliveryCandidate};
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::test]
async fn completed_blossom_enrichment_replaces_the_progress_candidate() {
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let (_, dispatch) = lock(&state).open(FeedSpec::Search("clip".to_owned()));
    let context = dispatch.expect("search dispatch").context;
    let author = Keys::generate();
    let digest = "a".repeat(64);
    let video = EventBuilder::new(Kind::Custom(22), "clip")
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url https://origin.example/{digest}.mp4"),
            "m video/mp4".to_owned(),
        ])
        .expect("imeta")])
        .sign_with_keys(&author)
        .expect("video");
    let servers = EventBuilder::new(Kind::Custom(10063), "")
        .tags([Tag::parse(["server", "https://blossom.example"]).expect("server")])
        .sign_with_keys(&author)
        .expect("server list");
    let (delivery, mut commands) = command_channel();
    let (sender, outcomes) = mpsc::unbounded_channel();
    let pump = tokio::spawn(pump_outcomes(
        OutcomeSinks {
            state,
            bootstrap: test_bootstrap().0,
            candidates: Some(delivery),
        },
        outcomes,
    ));

    sender
        .send(RetrievalOutcome::Progress {
            context: context.clone(),
            event: Box::new(video.clone()),
        })
        .expect("progress");
    assert_eq!(next(&mut commands).await.meta.urls.len(), 1);
    sender
        .send(RetrievalOutcome::Completed {
            context,
            result: Ok(vec![video, servers]),
            cursor: None,
            complete: true,
            purpose: RetrievalPurpose::Head,
        })
        .expect("completion");

    let enriched = next(&mut commands).await;
    assert_eq!(enriched.meta.sha256.as_deref(), Some(digest.as_str()));
    assert_eq!(
        enriched.meta.urls,
        [
            format!("https://origin.example/{digest}.mp4"),
            format!("https://blossom.example/{digest}"),
        ]
    );
    drop(sender);
    pump.await.expect("pump");
}

async fn next(commands: &mut CommandReceiver) -> DeliveryCandidate {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if let Some(candidate) = commands.try_candidate() {
                break candidate;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("candidate")
}
