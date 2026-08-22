use super::{engine_ranges, metadata_ranges};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::ByteRange;
use ghostr_partial_store::partial_range_store::{
    ContentRevision, StoredEvidenceId, StoredMediaSnapshot,
};

const TIMELINE_PARSER_PROFILE: u16 = 3;

#[derive(Clone)]
pub(crate) struct TimelineEvidence {
    binding: RepresentationBinding,
    revision: ContentRevision,
    total: u64,
    spans: Vec<ByteRange>,
    stored: StoredEvidenceId,
    parser_profile: u16,
}

impl std::fmt::Debug for TimelineEvidence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        })
    }

    pub(crate) fn binding(&self) -> &RepresentationBinding {
        &self.binding
    }

    pub(crate) fn revision(&self) -> ContentRevision {
        self.revision
    }

    pub(crate) fn total(&self) -> u64 {
        self.total
    }

    pub(crate) fn spans(&self) -> &[ByteRange] {
        &self.spans
    }

    pub(crate) fn same_parse(&self, other: &Self) -> bool {
        self.binding == other.binding
            && self.total == other.total
            && self.spans == other.spans
            && self.stored == other.stored
            && self.parser_profile == other.parser_profile
    }

    pub(crate) fn still_valid_in(&self, snapshot: &StoredMediaSnapshot) -> bool {
        let spans: Vec<_> = self.spans.iter().map(|span| span.start..span.end).collect();
        snapshot.binding() == Some(&self.binding)
            && snapshot.total_len() == Some(self.total)
            && self.parser_profile == TIMELINE_PARSER_PROFILE
            && snapshot.evidence_id_for(&spans) == Some(self.stored)
    }
}
