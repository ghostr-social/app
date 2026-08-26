use super::CatalogEntry;
use crate::evidence::{
    Confidence, Evidence, EvidenceAssessment, EvidenceScope, EvidenceSource, EvidenceValue,
};

const LEGACY_METADATA_CONFIDENCE_BPS: u16 = 7_500;

impl CatalogEntry {
    pub(super) fn seed_declared_evidence(&mut self) {
        let issuer = self.post.as_str().to_owned();
        let urls = self.meta.urls.clone();
        for url in urls {
            self.seed_url(&issuer, &url);
        }
    }

    pub fn evidence_assessment_for(&self, source: &str, now_ms: u64) -> EvidenceAssessment {
        self.ledger.assessment(source, now_ms)
    }

    pub(super) fn quarantine_integrity(&mut self, digest: &str, origin: &str, observed_at_ms: u64) {
        self.quarantined = true;
        self.timeline = None;
        self.tail_timeline_needed = false;
        self.ledger
            .quarantine_digest(digest, origin, observed_at_ms);
        self.evidence_clock_ms = self.evidence_clock_ms.max(observed_at_ms);
    }

    pub(super) fn record_nostr_metadata(
        &mut self,
        records: Vec<crate::evidence::NostrMetadataEvidence>,
    ) {
        if let Some(observed_at_ms) = records.iter().map(|item| item.observed_at_ms).max() {
            self.ledger
                .invalidate_nostr_issuer(self.post.as_str(), observed_at_ms);
        }
        for metadata in records {
            self.record_metadata_fields(&metadata);
            self.evidence_clock_ms = self.evidence_clock_ms.max(metadata.observed_at_ms);
        }
    }

    fn record_metadata_fields(&mut self, metadata: &crate::evidence::NostrMetadataEvidence) {
        let confidence = Confidence::new(LEGACY_METADATA_CONFIDENCE_BPS)
            .expect("legacy metadata confidence stays within the confidence scale");
        for url in &metadata.urls {
            for value in metadata.values() {
                self.ledger.record(Evidence::new(
                    value,
                    metadata.source(),
                    metadata.observed_at_ms,
                    confidence,
                    metadata.scope(url),
                ));
            }
        }
    }

    fn seed_url(&mut self, issuer: &str, url: &str) {
        if let Some(value) = self.meta.size_bytes {
            self.record_declared(issuer, url, EvidenceValue::SizeBytes(value));
        }
        if let Some(value) = self.meta.duration_ms {
            self.record_declared(issuer, url, EvidenceValue::DurationMs(value));
        }
        if let Some(value) = self.meta.sha256.clone() {
            self.record_declared(issuer, url, EvidenceValue::AdvertisedHash(value));
        }
    }

    fn record_declared(&mut self, issuer: &str, url: &str, value: EvidenceValue) {
        let confidence =
            Confidence::new(LEGACY_METADATA_CONFIDENCE_BPS).expect("legacy confidence is valid");
        self.ledger.record(Evidence::new(
            value,
            EvidenceSource::nostr(issuer),
            self.evidence_clock_ms,
            confidence,
            EvidenceScope::url(url),
        ));
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "ledger/test_support.rs"]
mod test_support;
