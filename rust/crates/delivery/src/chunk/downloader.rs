//! Ranged chunk downloader (plan Phase 1 step 5): fetches one granted
//! byte range with an HTTP `Range` GET, streams it into the partial
//! range store, and feeds the per-host performance model. Honors
//! cooperative cancellation so scroll-past can abandon transfers while
//! keeping the bytes already fetched.

use crate::chunk::cancel::CancelToken;
pub use crate::chunk::generation::OriginGeneration;
pub use crate::chunk::sink::ResponseWriteMode;
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use anyhow::Result;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;

mod captured;
mod opened;
mod outcome;
mod reply;
mod streamed;
mod telemetry;
mod transfer;
pub use crate::chunk::sink::{ChunkSink, ChunkWrite};
pub use crate::chunk::traffic::ChunkTraffic as DownloadTraffic;

/// One granted retrieval action for one URL.
pub struct ChunkSpec<'a> {
    pub client: &'a dyn MediaHttpRequests,
    pub url: &'a str,
    pub request: RetrievalRequest,
    pub continuation: Option<&'a SourceGeneration>,
    pub timeouts: TransferTimeouts,
}

/// What one chunk transfer accomplished.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChunkResult {
    pub bytes_written: u64,
    pub range_support: Option<bool>,
    pub range_ignored: bool,
    pub cancelled: bool,
    pub total_bytes: Option<u64>,
    pub promoted: bool,
    pub(crate) request_started: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseObservation {
    Partial {
        range: ByteRange,
        total: Option<u64>,
    },
    Body {
        request: RetrievalRequest,
        total: Option<u64>,
        range_support: Option<bool>,
        promoted: bool,
    },
    Ignored {
        total: Option<u64>,
        range_support: Option<bool>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenedResponse {
    observation: ResponseObservation,
    generation: Option<SourceGeneration>,
    mode: ResponseWriteMode,
    evidence: HttpResponseEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponseEvidence {
    pub final_url: String,
    pub content_type: Option<String>,
    pub validator: Option<EvidenceValidator>,
}

impl OpenedResponse {
    pub(crate) fn new(
        observation: ResponseObservation,
        generation: Option<SourceGeneration>,
        mode: ResponseWriteMode,
        evidence: HttpResponseEvidence,
    ) -> Self {
        Self {
            observation,
            generation,
            mode,
            evidence,
        }
    }

    pub fn observation(&self) -> ResponseObservation {
        self.observation
    }

    pub fn generation(&self) -> Option<&SourceGeneration> {
        self.generation.as_ref()
    }

    pub fn mode(&self) -> ResponseWriteMode {
        self.mode
    }

    pub fn evidence(&self) -> &HttpResponseEvidence {
        &self.evidence
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseAdmission {
    Proceed,
    Reject,
}

pub(crate) use captured::ObservedChunk;

/// Executes an admitted range and emits transfer observations.
pub async fn download_chunk_observed<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    sink: &W,
    stats: &mut HostStats,
    cancel: &CancelToken,
    network: &NetworkThrottle,
    traffic: &mut dyn ChunkTraffic,
) -> Result<ChunkResult> {
    download_chunk_captured(spec, sink, stats, cancel, network, traffic)
        .await
        .result
}

pub(crate) async fn download_chunk_captured<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    sink: &W,
    stats: &mut HostStats,
    cancel: &CancelToken,
    network: &NetworkThrottle,
    traffic: &mut dyn ChunkTraffic,
) -> ObservedChunk {
    captured::download(spec, sink, stats, cancel, network, traffic).await
}
