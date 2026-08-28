use super::StatsKeeper;
use crate::manager::inflight::InFlightChunks;
use crate::manager::transfers::{ChunkDone, ProbeObservation};
use core::time::Duration;
use fixture::{temp_directory, transfer_identity};
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::{ByteRange, ChunkId, PostId};
use ghostr_net::media_request_executor::MediaRequestAdmissionTimeout;

#[path = "admission_timeout_fixture.rs"]
mod fixture;

#[tokio::test]
async fn local_admission_timeout_does_not_train_remote_origin_failure() {
    let root = temp_directory("ghostr-admission-stats");
    let path = root.join("host_stats.json");
    let mut keeper = StatsKeeper::load(path.clone(), Duration::ZERO).await;
    note_chunk_timeout(&mut keeper);
    note_probe_timeout(&mut keeper);

    assert_eq!(keeper.stats().failure_ratio("chunk.example"), 0.0);
    assert_eq!(keeper.stats().failure_ratio("probe.example"), 0.0);
    assert_eq!(probe_samples(&keeper), 0.0);
    keeper.save_now().await;
    assert!(
        !path.exists(),
        "local queue exhaustion is not host evidence"
    );
    std::fs::remove_dir_all(root).expect("remove fixture");
}

fn note_chunk_timeout(keeper: &mut StatsKeeper) {
    let post = PostId::new("chunk");
    let url = "https://chunk.example/video.mp4";
    let mut inflight = InFlightChunks::new();
    let chunk = ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 1),
    };
    let attempt = inflight.next_attempt(chunk, transfer_identity(&post, url));
    keeper.note_chunk(&ChunkDone {
        attempt,
        url: url.to_owned(),
        outcome: Err(MediaRequestAdmissionTimeout.into()),
        received_bytes: 0,
        origin: None,
        open_body: None,
        request_started: false,
        whole_body_completion: None,
        response_evidence: None,
    });
}

fn note_probe_timeout(keeper: &mut StatsKeeper) {
    keeper.note_probe(&ProbeObservation {
        post: PostId::new("probe"),
        url: "https://probe.example/video.mp4".to_owned(),
        outcome: Err(MediaRequestAdmissionTimeout.into()),
        attempt_context: None,
    });
}

fn probe_samples(keeper: &StatsKeeper) -> f64 {
    let now = crate::manager::time::unix_time_ms();
    let context = OriginContext::new(RequestMethod::Head, 0, MediaClass::Unknown)
        .with_network(NetworkClass::Unavailable)
        .with_concurrency(1)
        .with_observed_at_ms(now);
    let query = OriginQuery::new("https://probe.example/video.mp4", context);
    keeper
        .stats()
        .origin_model()
        .estimate(&query, now, DecisionMode::Normal)
        .effective_samples
}
