use super::evidence::StoredEvidence;
use crate::partial_range_store::ContentRevision;
use ghostr_engine::representation::RepresentationBinding;
use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredMediaSnapshot {
    pub(super) binding: Option<RepresentationBinding>,
    pub(super) revision: ContentRevision,
    pub(super) total_len: Option<u64>,
    pub(super) ranges: Vec<Range<u64>>,
    pub(super) planning_ranges: Vec<Range<u64>>,
    pub(super) complete: bool,
    pub(super) finalized: bool,
    pub(super) continuation_source: Option<String>,
    pub(super) evidence: StoredEvidence,
}

impl StoredMediaSnapshot {
    pub fn binding(&self) -> Option<&RepresentationBinding> {
        self.binding.as_ref()
    }

    pub fn revision(&self) -> ContentRevision {
        self.revision
    }

    pub fn total_len(&self) -> Option<u64> {
        self.total_len
    }

    pub fn ranges(&self) -> &[Range<u64>] {
        &self.ranges
    }

    pub fn planning_ranges(&self) -> &[Range<u64>] {
        &self.planning_ranges
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    pub fn continuation_source(&self) -> Option<&str> {
        self.continuation_source.as_deref()
    }
}
