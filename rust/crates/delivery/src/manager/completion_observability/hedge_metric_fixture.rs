use crate::chunk::downloader::ChunkResult;
use crate::delivery_events::DecisionResolution;
use crate::manager::inflight::{ChunkAttempt, InFlightChunks};
use crate::manager::transfers::ChunkDone;
use ghostr_engine::adaptive::{DecisionAction, WholeBodyContract};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};

pub(crate) fn done(bytes: u64, cancelled: bool) -> ChunkDone {
    ChunkDone {
        attempt: attempt(),
        url: "https://primary.example/video.mp4".into(),
        outcome: Ok(ChunkResult {
            bytes_written: bytes,
            range_support: Some(true),
            range_ignored: false,
            cancelled,
            total_bytes: Some(64),
            promoted: false,
            request_started: true,
        }),
        received_bytes: bytes,
        origin: None,
        request_started: true,
        whole_body_completion: None,
        response_evidence: None,
    }
}

pub(crate) fn failed() -> ChunkDone {
    ChunkDone {
        attempt: attempt(),
        url: "https://alternate.example/video.mp4".into(),
        outcome: Err(anyhow::anyhow!("alternate failed")),
        received_bytes: 0,
        origin: None,
        request_started: true,
        whole_body_completion: None,
        response_evidence: None,
    }
}

pub(crate) fn policy_limited() -> ChunkDone {
    let error = crate::chunk::whole_body_limit::WholeBodyLimitReached::check(
        8,
        1,
        WholeBodyContract::Capped { maximum_bytes: 8 },
    )
    .unwrap_err();
    ChunkDone {
        attempt: attempt(),
        url: "https://primary.example/video.mp4".into(),
        outcome: Err(error),
        received_bytes: 9,
        origin: None,
        request_started: true,
        whole_body_completion: None,
        response_evidence: None,
    }
}

pub(super) fn resolution(request: &str) -> DecisionResolution {
    DecisionResolution {
        action: DecisionAction {
            post_id: "post".into(),
            source_id: "source".into(),
            request: request.into(),
            bytes_start: 0,
            bytes_end: 64,
            expected_playable_gain_ms: 1,
            utility_micros: 1,
            reason: "CurrentStallPrevention".into(),
            retained: false,
        },
        warp_action: None,
        elapsed_ms: 1,
    }
}

fn attempt() -> ChunkAttempt {
    let post = PostId::new("post");
    let url = "https://primary.example/video.mp4";
    let mut catalog = Catalog::new();
    catalog.upsert(
        post.clone(),
        VideoMeta {
            urls: vec![url.into()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(64),
            duration_ms: Some(1),
        },
    );
    let identity = catalog.transfer_identity(&post, url).unwrap();
    let chunk = ChunkId {
        post,
        range: ByteRange::new(0, 64),
    };
    InFlightChunks::new().next_attempt(chunk, identity)
}
