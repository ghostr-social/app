use super::StatsKeeper;
use crate::chunk::downloader::ChunkResult;
use crate::manager::inflight::InFlightChunks;
use crate::manager::transfers::ChunkDone;
use core::time::Duration;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, OpenBodyObservation, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::{ByteRange, ChunkId, PostId};

const URL: &str = "https://body.example/video.mp4";
const AT_MS: u64 = 1_000;

#[tokio::test]
async fn manager_owned_open_body_evidence_is_persisted() {
    let root = crate::tests::support::temp_directory("ghostr-open-body-stats");
    let path = root.join("host_stats.json");
    let mut keeper = StatsKeeper::load(path.clone(), Duration::ZERO).await;
    keeper.note_chunk(&done());
    keeper.save_now().await;

    let json = std::fs::read_to_string(&path).expect("persisted stats");
    let restored = HostStats::from_json(&json).expect("valid persisted stats");
    let estimate =
        restored
            .origin_model()
            .estimate_open_body(&query(), AT_MS, DecisionMode::Normal);
    assert!(estimate.effective_samples > 0.0);
    std::fs::remove_dir_all(root).expect("remove fixture");
}

fn done() -> ChunkDone {
    let post = PostId::new("clip");
    let chunk = ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 100_000),
    };
    let mut inflight = InFlightChunks::new();
    let identity = crate::tests::support::transfer_identity(&post, URL);
    ChunkDone {
        attempt: inflight.next_attempt(chunk, identity),
        url: URL.to_owned(),
        outcome: Ok(success()),
        received_bytes: 100_000,
        origin: None,
        open_body: Some(Box::new(OpenBodyObservation::success(query(), AT_MS))),
        request_started: true,
        whole_body_completion: None,
        response_evidence: None,
    }
}

fn success() -> ChunkResult {
    ChunkResult {
        bytes_written: 100_000,
        range_support: Some(false),
        range_ignored: true,
        cancelled: false,
        total_bytes: Some(100_000),
        promoted: true,
        request_started: true,
    }
}

fn query() -> OriginQuery {
    OriginQuery::new(
        URL,
        OriginContext::new(RequestMethod::RangeGet, 100_000, MediaClass::ProgressiveMp4)
            .with_observed_at_ms(AT_MS),
    )
}
