use super::StoredMediaSnapshot;
use crate::partial_range_manifest::{IntervalChecksum, RangeManifest};
use core::ops::Range;
use sha2::{Digest as _, Sha256};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct StoredEvidenceId([u8; 32]);

impl core::fmt::Debug for StoredEvidenceId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("StoredEvidenceId(<redacted>)")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StoredEvidence {
    intervals: Vec<IntervalChecksum>,
}

impl StoredEvidence {
    pub(super) fn capture(manifest: &RangeManifest) -> Self {
        Self {
            intervals: manifest.checksum_records().to_vec(),
        }
    }
}

impl StoredMediaSnapshot {
    /// Exact local-byte evidence for the selected readable spans.
    pub fn evidence_id_for(&self, spans: &[Range<u64>]) -> Option<StoredEvidenceId> {
        let mut digest = Sha256::new();
        digest.update(b"ghostr-stored-evidence-v1");
        digest.update([u8::from(self.total_len.is_some())]);
        digest.update(self.total_len.unwrap_or_default().to_be_bytes());
        digest.update((spans.len() as u64).to_be_bytes());
        for span in spans {
            hash_span(&mut digest, span, &self.evidence.intervals)?;
        }
        Some(StoredEvidenceId(digest.finalize().into()))
    }
}

fn hash_span(digest: &mut Sha256, span: &Range<u64>, records: &[IntervalChecksum]) -> Option<()> {
    if span.start >= span.end {
        return None;
    }
    digest.update(span.start.to_be_bytes());
    digest.update(span.end.to_be_bytes());
    let record_count = records
        .iter()
        .filter(|record| overlaps(record, span))
        .count();
    digest.update((record_count as u64).to_be_bytes());
    let mut cursor = span.start;
    for record in records.iter().filter(|record| overlaps(record, span)) {
        let record_span = record.span();
        if record_span.start > cursor {
            return None;
        }
        digest.update(record_span.start.to_be_bytes());
        digest.update(record_span.end.to_be_bytes());
        digest.update(record.digest().as_bytes());
        cursor = cursor.max(record_span.end);
    }
    (cursor >= span.end).then_some(())
}

fn overlaps(record: &IntervalChecksum, span: &Range<u64>) -> bool {
    let record = record.span();
    record.start < span.end && span.start < record.end
}
