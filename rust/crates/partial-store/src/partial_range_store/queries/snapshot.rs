use super::evidence::StoredEvidence;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_store::single_response::SingleResponseStorage;
use crate::partial_range_store::ContentRevision;
use core::ops::Range;
use ghostr_engine::representation::RepresentationBinding;

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

pub(super) struct SnapshotCapture {
    total_len: Option<u64>,
    stable_ranges: Vec<Range<u64>>,
    ranges: Vec<Range<u64>>,
    complete: bool,
    finalized: bool,
    evidence: StoredEvidence,
}

pub(super) struct SnapshotProjection {
    pub(super) binding: Option<RepresentationBinding>,
    pub(super) revision: ContentRevision,
    pub(super) planning_ranges: Vec<Range<u64>>,
    pub(super) continuation_source: Option<String>,
}

impl SnapshotCapture {
    pub(super) fn new(stable: &RangeManifest, readable: &RangeManifest, finalized: bool) -> Self {
        Self {
            total_len: stable.total_len(),
            stable_ranges: stable.ranges(),
            ranges: readable.ranges(),
            complete: finalized || stable.is_complete(),
            finalized,
            evidence: StoredEvidence::capture(readable),
        }
    }

    pub(super) fn planning_ranges(
        &self,
        storage: Option<SingleResponseStorage>,
    ) -> Vec<Range<u64>> {
        match storage {
            Some(SingleResponseStorage::Live { .. } | SingleResponseStorage::Memory) => Vec::new(),
            _ => self.stable_ranges.clone(),
        }
    }

    pub(super) const fn complete(&self) -> bool {
        self.complete
    }

    pub(super) fn into_snapshot(self, projection: SnapshotProjection) -> StoredMediaSnapshot {
        StoredMediaSnapshot {
            binding: projection.binding,
            revision: projection.revision,
            total_len: self.total_len,
            ranges: self.ranges,
            planning_ranges: projection.planning_ranges,
            complete: self.complete,
            finalized: self.finalized,
            continuation_source: projection.continuation_source,
            evidence: self.evidence,
        }
    }
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
