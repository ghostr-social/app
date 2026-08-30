use super::{LoopReopenFixture, BODY};
use crate::gateway_fixture::progressive::{progressive_harness, ProgressiveHarness};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease, SourceGeneration,
};
use ghostr_engine::ByteRange;
use ghostr_partial_store::partial_range_store::ResponseOpenResult;

const SOURCE: &str = "https://cdn.example/clip.mp4";

pub async fn seeded_harness() -> LoopReopenFixture {
    let harness = progressive_harness("ghostr-progressive-loop-reopen");
    harness.posts.insert("clip");
    harness
        .bind_video("clip", SOURCE, Some(BODY.len() as u64))
        .await;
    let binding = harness
        .store
        .representation_binding("clip")
        .await
        .expect("binding");
    let transfer = binding.transfer(SOURCE).expect("transfer");
    let generation = generation();
    harness
        .store
        .apply_http_generation(
            &transfer,
            HttpGenerationAuthority::Trusted(generation.clone()),
        )
        .await
        .expect("HTTP generation");
    let source = SourceGeneration::try_new(SOURCE, "\"same\"", BODY.len() as u64)
        .expect("source generation");
    seed_sparse_prefix(&harness, &transfer, source).await;
    LoopReopenFixture {
        harness,
        transfer,
        generation,
    }
}

pub async fn serve(
    harness: &ProgressiveHarness,
    capability: &str,
) -> (String, tokio::task::JoinHandle<Result<(), std::io::Error>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("address");
    let router = harness.router.clone();
    let server = tokio::spawn(async move { axum::serve(listener, router).await });
    let url = format!("http://{address}/video.mp4?id=clip&cap={capability}");
    (url, server)
}

async fn seed_sparse_prefix(
    harness: &ProgressiveHarness,
    transfer: &ghostr_engine::representation::TransferIdentity,
    source: SourceGeneration,
) {
    let store = &harness.store;
    let action = store
        .reserve_action(transfer, 1, 2)
        .await
        .expect("sparse reserve");
    let opened = store
        .open_sparse_response(transfer, &action, source.clone(), ByteRange::new(0, 2))
        .await
        .expect("sparse response open");
    assert_eq!(opened, ResponseOpenResult::Opened);
    let wrote = store
        .write_range_for_action_if_current(transfer, &source, &action, 0, &BODY[..2])
        .await
        .expect("sparse prefix");
    assert!(wrote);
    assert!(store
        .finish_sparse_response(transfer, &source, &action)
        .await
        .expect("sparse finish"));
    store.release_action(&action).await;
}

fn generation() -> HttpGenerationLease {
    let validator = EvidenceValidator::strong_etag("\"same\"").expect("strong ETag");
    let key = HttpGenerationKey::try_new(SOURCE, Some(validator)).expect("generation key");
    HttpGenerationLease::try_new(key, 1).expect("generation lease")
}
