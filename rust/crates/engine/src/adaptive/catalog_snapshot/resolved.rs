use super::super::sources::best_available;
use super::CandidateEvidence;
use crate::adaptive::{CandidateSnapshot, MediaLayout, OriginHealth, PlayableRange};
use crate::catalog::CatalogEntry;
use crate::evidence::{EvidenceAssessment, EvidenceValue};
use crate::ByteRange;
use std::collections::HashSet;

mod resolution;
pub(super) use resolution::resolve;

pub(super) struct ResolvedCandidate {
    preferred_source: Option<String>,
    assessment: EvidenceAssessment,
    total: Option<u64>,
    present: Vec<ByteRange>,
    bitrate: u64,
    duration: u64,
    layout: MediaLayout,
    startup: Option<crate::media_timeline::StartupFootprint>,
    playable_ranges: Vec<PlayableRange>,
    timeline_probe: Option<PlayableRange>,
}

impl ResolvedCandidate {
    pub(super) fn into_snapshot(self, evidence: CandidateEvidence) -> CandidateSnapshot {
        CandidateSnapshot {
            post: evidence.post,
            feed_offset: evidence.feed_offset,
            view_probability: evidence.view_probability,
            retrieval_eligible: true,
            total_bytes: self.total,
            bitrate_bps: self.bitrate,
            duration_ms: self.duration,
            layout: self.layout,
            preferred_source: self.preferred_source,
            startup: self.startup,
            player_preparation: crate::adaptive::PlayerPreparation::Unverified,
            direct_playback_blocked: false,
            timeline_probe: self.timeline_probe,
            playable_ranges: self.playable_ranges,
            demanded: None,
            present: self.present,
            finalized: false,
            recently_evicted: evidence.recently_evicted,
            in_flight: evidence.in_flight,
            origins: evidence.origins,
            evidence: self.assessment,
        }
    }
}

fn timeline_probe(
    entry: &CatalogEntry,
    layout: MediaLayout,
    playable: &[PlayableRange],
) -> Option<PlayableRange> {
    (layout == MediaLayout::Streamable && entry.needs_timeline_probe())
        .then(|| playable.last().copied())
        .flatten()
}

fn selected_source<'a>(
    entry: &'a CatalogEntry,
    origins: &'a [OriginHealth],
    continuation: Option<&'a str>,
) -> Option<&'a str> {
    if let Some(source) = continuation.filter(|source| source_available(entry, origins, source)) {
        return Some(source);
    }
    best_available(origins)
        .map(|origin| origin.source.as_str())
        .or_else(|| entry.meta.urls.first().map(String::as_str))
}

fn source_available(entry: &CatalogEntry, origins: &[OriginHealth], source: &str) -> bool {
    let known = entry.meta.urls.iter().any(|url| url == source);
    known
        && (origins.is_empty()
            || origins
                .iter()
                .any(|origin| origin.source == source && origin.available))
}

fn source_compatible(selected: Option<&str>, continuation: Option<&str>) -> bool {
    continuation.is_none() || selected == continuation
}

fn effective_total(
    assessment: &EvidenceAssessment,
    stored: Option<u64>,
    compatible: bool,
) -> Option<u64> {
    compatible
        .then_some(stored)
        .flatten()
        .or(assessment.size.exact)
        .filter(|total| *total > 0)
}

fn layout(
    assessment: &EvidenceAssessment,
    source: Option<&str>,
    independent: &HashSet<String>,
) -> MediaLayout {
    if source.is_some_and(|source| independent.contains(source)) {
        return MediaLayout::RequiresCompleteFile;
    }
    match assessment.value(crate::evidence::EvidenceField::RangeSupport) {
        Some(EvidenceValue::RangeSupport(true)) => MediaLayout::Streamable,
        Some(EvidenceValue::RangeSupport(false)) => MediaLayout::RequiresCompleteFile,
        _ => MediaLayout::Unknown,
    }
}

fn duration_ms(entry: &CatalogEntry, total: Option<u64>, bitrate: u64) -> u64 {
    entry
        .meta
        .duration_ms
        .filter(|duration| *duration > 0)
        .unwrap_or_else(|| {
            total
                .unwrap_or(crate::adaptive::allocation::REQUEST_SLICE_BYTES)
                .saturating_mul(8_000)
                .div_ceil(bitrate)
        })
}
