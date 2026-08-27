use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::ChunkResult;
use crate::manager::inflight::{ActionRegistration, InFlightChunks};
use crate::tests::support::{chunk_request, transfer_identity};
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::origin_model::{
    AdmissionClaim, ErrorReason, MediaClass, OriginContext, OriginModel, OriginObservation,
    OriginQuery, RequestMethod,
};
use ghostr_engine::{ByteRange, ChunkId, PostId};

pub(super) const URL: &str = "https://recovered.example/video.mp4";

pub(super) fn finished_action(
    claim: AdmissionClaim,
) -> (
    crate::manager::inflight::ChunkAttempt,
    crate::manager::inflight::FinishedAction,
) {
    let post = PostId::new("post");
    let range = ByteRange::new(0, 65_536);
    let chunk = ChunkId {
        post: post.clone(),
        range,
    };
    let identity = transfer_identity(&post, URL);
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(chunk.clone(), identity);
    let (handle, _) = cancel_pair();
    active.insert_action(ActionRegistration {
        attempt: &attempt,
        priority: chunk_request(chunk, PreemptionAuthority::Transition),
        retrieval: RetrievalRequest::FetchRange {
            bytes: range,
            promotion: None,
        },
        host: "recovered.example".into(),
        committed_until_ms: 0,
        launched_at_ms: 0,
        handle,
        store_action: None,
        committed_network_bytes: None,
        admission_claim: Some(claim),
    });
    let finished = active.finish_with_resources(&attempt);
    (attempt, finished)
}

pub(super) fn query(method: RequestMethod, bytes: u64) -> OriginQuery {
    OriginQuery::new(
        URL,
        OriginContext::new(method, bytes, MediaClass::WholeObject),
    )
}

pub(super) fn open_circuit(query: &OriginQuery) -> OriginModel {
    let mut model = OriginModel::default();
    for at_ms in 1_000..=1_002 {
        model.observe(&OriginObservation::failure(
            query.clone(),
            at_ms,
            ErrorReason::Timeout,
        ));
    }
    model
}

pub(super) fn success() -> ChunkResult {
    ChunkResult {
        bytes_written: 65_536,
        range_support: Some(true),
        range_ignored: false,
        cancelled: false,
        total_bytes: Some(900_000),
        promoted: false,
        request_started: true,
    }
}
