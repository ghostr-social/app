use super::super::{SegmentedCache, StageReservation, StagedObject};
use ghostr_engine::PostId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StageRequest {
    pub(super) url: String,
    pub(super) offset: u64,
    pub(super) block_bytes: u64,
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
    pub(super) generation: u64,
    attempt: u64,
    pub(super) request: StageRequest,
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
    pub(super) post: PostId,
    pub(super) fence: StageFence,
}

pub(crate) struct InflightStage {
    pub(super) prefix: Option<(usize, StagedObject)>,
    pub(crate) reserved_bytes: u64,
}

pub(crate) struct StageLease {
    pub(super) cache: SegmentedCache,
    pub(super) key: Option<InflightKey>,
}
