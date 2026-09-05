use super::{engine_ranges, metadata_ranges};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::ByteRange;
use ghostr_partial_store::partial_range_store::{
    ContentRevision, StoredEvidenceId, StoredMediaSnapshot,
};

const TIMELINE_PARSER_PROFILE: u16 = 4;

pub(super) type TimelineIndexSource = (
    ghostr_engine::representation::TransferIdentity,
    ghostr_engine::representation::SourceGeneration,
);

#[derive(Clone)]
pub(crate) struct TimelineEvidence {
    binding: RepresentationBinding,
    revision: ContentRevision,
    total: u64,
    spans: Vec<ByteRange>,
    stored: StoredEvidenceId,
    parser_profile: u16,
    pub(super) source: Option<TimelineIndexSource>,
}

impl core::fmt::Debug for TimelineEvidence {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("TimelineEvidence")
            .field("post", &self.binding.post())
            .field("revision", &self.revision)
            .field("total", &self.total)
            .field("spans", &self.spans)
            .field("parser_profile", &self.parser_profile)
            .finish_non_exhaustive()
    }
}

impl TimelineEvidence {
    pub(crate) fn from_snapshot(
        binding: &RepresentationBinding,
        snapshot: &StoredMediaSnapshot,
    ) -> Option<Self> {
        if snapshot.binding() != Some(binding) {
            return None;
        }
        let total = snapshot.total_len()?;
        let spans = metadata_ranges(total, &engine_ranges(snapshot.ranges()));
        let stored_spans: Vec<_> = spans.iter().map(|span| span.start..span.end).collect();
        let stored = snapshot.evidence_id_for(&stored_spans)?;
        Some(Self {
            binding: binding.clone(),
            revision: snapshot.revision(),
            total,
            spans,
            stored,
            parser_profile: TIMELINE_PARSER_PROFILE,
            source: None,
        })
    }

    pub(super) fn binding(&self) -> &RepresentationBinding {
        &self.binding
    }

    pub(super) fn revision(&self) -> ContentRevision {
        self.revision
    }

    pub(super) fn total(&self) -> u64 {
        self.total
    }

    pub(super) fn spans(&self) -> &[ByteRange] {
        &self.spans
    }

    pub(super) fn same_parse(&self, other: &Self) -> bool {
        self.binding == other.binding
            && self.total == other.total
            && self.spans == other.spans
            && self.stored == other.stored
            && self.parser_profile == other.parser_profile
            && self.source == other.source
    }

    pub(super) fn still_valid_in(&self, snapshot: &StoredMediaSnapshot) -> bool {
        let spans: Vec<_> = self.spans.iter().map(|span| span.start..span.end).collect();
        snapshot.binding() == Some(&self.binding)
            && snapshot.total_len() == Some(self.total)
            && self.parser_profile == TIMELINE_PARSER_PROFILE
            && snapshot.evidence_id_for(&spans) == Some(self.stored)
    }
}
