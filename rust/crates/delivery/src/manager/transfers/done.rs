use super::ChunkAttempt;
use crate::chunk::downloader::{ChunkResult, HttpResponseEvidence};
use crate::chunk::traffic::WholeBodyCompletion;
use ghostr_engine::origin_model::{OpenBodyObservation, OriginObservation};

pub(crate) struct ChunkDone {
    pub attempt: ChunkAttempt,
    pub url: String,
    pub outcome: anyhow::Result<ChunkResult>,
    pub received_bytes: u64,
    pub origin: Option<Box<OriginObservation>>,
    pub open_body: Option<Box<OpenBodyObservation>>,
    pub request_started: bool,
    pub whole_body_completion: Option<WholeBodyCompletion>,
    pub response_evidence: Option<HttpResponseEvidence>,
}
