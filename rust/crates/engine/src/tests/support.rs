use crate::adaptive::{AllocationPlan, CandidateSnapshot, PlayableRange};
use crate::evidence::{
    Confidence, Evidence, EvidenceLedger, EvidenceScope, EvidenceSource, EvidenceValue,
};
use crate::{ByteRange, DeliveryKind, PostId, VideoMeta};

pub(super) fn set_reliable_total_bytes(
    candidate: &mut CandidateSnapshot,
    bytes: u64,
    observed_at_ms: u64,
) {
    let source = candidate
        .preferred_source
        .clone()
        .or_else(|| {
            candidate
                .origins
                .iter()
                .find(|origin| origin.available)
                .map(|origin| origin.source.clone())
        })
        .expect("test candidate source");
    set_reliable_total_bytes_for_source(candidate, bytes, observed_at_ms, &source);
}

pub(super) fn set_reliable_total_bytes_for_source(
    candidate: &mut CandidateSnapshot,
    bytes: u64,
    observed_at_ms: u64,
    source: &str,
) {
    let mut ledger = EvidenceLedger::default();
    ledger.record(Evidence::new(
        EvidenceValue::SizeBytes(bytes),
        EvidenceSource::response(source),
        observed_at_ms,
        Confidence::new(9_000).expect("valid test confidence"),
        EvidenceScope::url(source),
    ));
    candidate.total_bytes = Some(bytes);
    candidate.evidence = ledger.assessment(source, observed_at_ms);
}

pub(super) fn planned_playable_ms(plan: &AllocationPlan, post: &PostId) -> u64 {
    plan.allocations
        .iter()
        .filter(|work| &work.post == post)
        .map(|work| work.expected_playable_gain_ms)
        .sum()
}

pub(super) fn playable_range(index: u64) -> PlayableRange {
    PlayableRange {
        bytes: ByteRange::new(index * 250_000, (index + 1) * 250_000),
        playable_ms: 2_000,
    }
}

pub(super) fn progressive_meta(size_bytes: Option<u64>, duration_ms: Option<u64>) -> VideoMeta {
    VideoMeta {
        urls: vec!["https://host.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes,
        duration_ms,
    }
}

pub(super) fn ids(raw: &[&str]) -> Vec<PostId> {
    raw.iter().map(|value| PostId::new(*value)).collect()
}
