use super::super::{SegmentedCache, StageReservation, StagedObject};
use ghostr_engine::PostId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StageRequest {
    pub(crate) url: String,
    pub(crate) offset: u64,
    pub(crate) block_bytes: u64,
}

impl StageRequest {
    pub(crate) const fn new(url: String, offset: u64, block_bytes: u64) -> Self {
        Self {
            url,
            offset,
            block_bytes,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StageFence {
    pub(crate) generation: u64,
    pub(crate) attempt: u64,
    pub(crate) request: StageRequest,
}

impl StageFence {
    pub(crate) const fn new(generation: u64, attempt: u64, request: StageRequest) -> Self {
        Self {
            generation,
            attempt,
            request,
        }
    }
}

pub(crate) struct StageAdmission {
    pub(super) post: PostId,
    pub(super) fence: StageFence,
    pub(super) eta_ms: u64,
    pub(super) reservation: StageReservation,
}

impl StageAdmission {
    pub(crate) const fn new(
        post: PostId,
        fence: StageFence,
        eta_ms: u64,
        reservation: StageReservation,
    ) -> Self {
        Self {
            post,
            fence,
            eta_ms,
            reservation,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct InflightKey {
    pub(crate) post: PostId,
    pub(crate) fence: StageFence,
}

pub(crate) struct InflightStage {
    pub(crate) prefix: Option<(usize, StagedObject)>,
    pub(crate) reserved_bytes: u64,
}

pub(crate) struct StageLease {
    pub(crate) cache: SegmentedCache,
    pub(crate) key: Option<InflightKey>,
}
