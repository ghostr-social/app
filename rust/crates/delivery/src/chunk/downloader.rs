//! Ranged chunk downloader (plan Phase 1 step 5).
//!
//! Each grant fetches one HTTP byte range, streams it into the partial store, and updates the host
//! model. Cooperative cancellation preserves bytes received before a post scrolls out of demand.

use crate::chunk::cancel::CancelToken;
pub use crate::chunk::generation::OriginGeneration;
pub use crate::chunk::sink::ResponseWriteMode;
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::MediaRequestExecutor;
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
    pub requests: &'a MediaRequestExecutor,
    pub url: &'a str,
    pub request: RetrievalRequest,
    pub priority: PreemptionAuthority,
    pub continuation: Option<&'a SourceGeneration>,
    pub timeouts: TransferTimeouts,
}

pub struct ChunkExecution<'a, W: ChunkWrite + ?Sized> {
    pub sink: &'a W,
    pub stats: &'a mut HostStats,
    pub cancel: &'a CancelToken,
    pub network: &'a NetworkThrottle,
    pub traffic: &'a mut dyn ChunkTraffic,
    pub network_class: ghostr_engine::origin_model::NetworkClass,
}

/// What one chunk transfer accomplished.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChunkResult {
    pub bytes_written: u64,
    pub range_support: Option<bool>,
    pub range_ignored: bool,
    pub cancelled: bool,
    pub total_bytes: Option<u64>,
    pub(crate) promoted: bool,
    pub(crate) request_started: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseObservation {
    Rejected(ResponseRejection),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseRejection {
    Status,
    ContentEncoding,
    MediaType,
    Semantics,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResponseFailure {
    RangeNoncompliant,
    InvalidResponse,
}

impl core::fmt::Display for ResponseFailure {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::RangeNoncompliant => formatter.write_str("range response is noncompliant"),
            Self::InvalidResponse => formatter.write_str("origin response is invalid"),
        }
    }
}

impl core::error::Error for ResponseFailure {}

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
    pub status: u16,
    pub content_type: Option<String>,
    pub validator: Option<EvidenceValidator>,
    pub observed: ghostr_engine::evidence::EvidenceTime,
}

impl HttpResponseEvidence {
    fn provenance_only(mut self) -> Self {
        self.content_type = None;
        self.validator = None;
        self
    }

    fn authority_only(mut self) -> Self {
        self.content_type = None;
        self
    }
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

    pub(crate) fn generation(&self) -> Option<&SourceGeneration> {
        self.generation.as_ref()
    }

    pub(crate) fn mode(&self) -> ResponseWriteMode {
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

pub use captured::ObservedChunk;

/// Executes an admitted range and emits transfer observations.
///
/// # Errors
///
/// Returns an error when admission, transport, response validation, or storage fails.
pub async fn download_chunk_observed<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    execution: ChunkExecution<'_, W>,
) -> ObservedChunk {
    captured::download(spec, execution).await
}
