use super::sources::best_available;
use super::{
    CandidateSnapshot, FeedOffset, InFlightAction, MediaLayout, OriginHealth, PlayableRange,
    ViewProbability,
};
use crate::catalog::{Catalog, CatalogEntry};
use crate::evidence::{EvidenceAssessment, EvidenceValue};
use crate::{ByteRange, EngineParams, PostId};
use std::collections::HashSet;

mod playable;
mod startup;

pub struct CandidateEvidence {
    pub post: PostId,
    pub feed_offset: FeedOffset,
    pub view_probability: ViewProbability,
    pub present: Vec<ByteRange>,
    pub stored_total: Option<u64>,
    pub continuation_source: Option<String>,
    pub independent_object_sources: HashSet<String>,
    pub recently_evicted: Vec<ByteRange>,
    pub in_flight: Vec<InFlightAction>,
    pub origins: Vec<OriginHealth>,
}

pub fn candidate_snapshot(
    catalog: &Catalog,
    params: &EngineParams,
    evidence: CandidateEvidence,
) -> Option<CandidateSnapshot> {
    candidate_snapshot_at(catalog, params, evidence, 0)
}

pub fn candidate_snapshot_at(
    catalog: &Catalog,
    params: &EngineParams,
    evidence: CandidateEvidence,
    observed_at_ms: u64,
) -> Option<CandidateSnapshot> {
    let entry = catalog.lookup(&evidence.post)?;
    if entry.is_quarantined() {
        return None;
    }
    let source = selected_source(
        entry,
        &evidence.origins,
        evidence.continuation_source.as_deref(),
    );
    let compatible = source_compatible(source, evidence.continuation_source.as_deref());
    let assessment = source
        .map(|source| entry.evidence_assessment_for(source, observed_at_ms))
        .unwrap_or_default();
    let total = effective_total(&assessment, evidence.stored_total, compatible);
    let present = if compatible {
        evidence.present
    } else {
        Vec::new()
    };
    let bitrate = catalog
        .estimated_bitrate_for_total(&evidence.post, total, params)
        .max(1);
    let duration = duration_ms(entry, total, bitrate);
    let layout = layout(&assessment, source, &evidence.independent_object_sources);
    let startup = startup::footprint(startup::Inputs {
        entry,
        layout,
        total,
        duration_ms: duration,
        present: &present,
    });
    let playable_ranges = playable::ranges(
        layout,
        playable::Inputs {
            entry,
            total,
            duration_ms: duration,
            chunk_bytes: params.chunk_bytes,
            present: &present,
        },
    );
    Some(CandidateSnapshot {
        post: evidence.post,
        feed_offset: evidence.feed_offset,
        view_probability: evidence.view_probability,
        retrieval_eligible: true,
        total_bytes: total,
        bitrate_bps: bitrate,
        duration_ms: duration,
        layout,
        preferred_source: source.map(str::to_owned),
        startup,
        player_preparation: super::PlayerPreparation::Unverified,
        timeline_probe: timeline_probe(entry, layout, &playable_ranges),
        playable_ranges,
        demanded: None,
        present,
        finalized: false,
        recently_evicted: evidence.recently_evicted,
        in_flight: evidence.in_flight,
        origins: evidence.origins,
        evidence: assessment,
    })
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
                .unwrap_or(super::allocation::REQUEST_SLICE_BYTES)
                .saturating_mul(8_000)
                .div_ceil(bitrate)
        })
}
