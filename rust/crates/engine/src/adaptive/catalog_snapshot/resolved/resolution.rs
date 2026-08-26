use super::{
    duration_ms, effective_total, layout, selected_source, source_compatible, timeline_probe,
    ResolvedCandidate,
};
use crate::adaptive::{MediaLayout, PlayableRange};
use crate::catalog::{Catalog, CatalogEntry};
use crate::evidence::EvidenceAssessment;
use crate::{ByteRange, EngineParams};

use super::super::{playable, startup, CandidateEvidence};

struct ResolveInput<'a> {
    catalog: &'a Catalog,
    params: &'a EngineParams,
    evidence: &'a CandidateEvidence,
    entry: &'a CatalogEntry,
}

struct SourceResolution<'a> {
    source: Option<&'a str>,
    assessment: EvidenceAssessment,
    total: Option<u64>,
    present: Vec<ByteRange>,
    layout: MediaLayout,
}

struct MediaGeometry {
    bitrate: u64,
    duration: u64,
    startup: Option<crate::media_timeline::StartupFootprint>,
    playable_ranges: Vec<PlayableRange>,
    timeline_probe: Option<PlayableRange>,
}

pub(in crate::adaptive::catalog_snapshot) fn resolve(
    catalog: &Catalog,
    params: &EngineParams,
    evidence: &CandidateEvidence,
    observed_at_ms: u64,
) -> Option<ResolvedCandidate> {
    let entry = catalog.lookup(&evidence.post)?;
    (!entry.is_quarantined()).then_some(())?;
    let input = ResolveInput {
        catalog,
        params,
        evidence,
        entry,
    };
    let source = source_resolution(&input, observed_at_ms);
    let geometry = media_geometry(&input, &source);
    Some(ResolvedCandidate::from_parts(source, geometry))
}

impl ResolvedCandidate {
    fn from_parts(source: SourceResolution<'_>, geometry: MediaGeometry) -> Self {
        Self {
            preferred_source: source.source.map(str::to_owned),
            assessment: source.assessment,
            total: source.total,
            present: source.present,
            bitrate: geometry.bitrate,
            duration: geometry.duration,
            layout: source.layout,
            startup: geometry.startup,
            playable_ranges: geometry.playable_ranges,
            timeline_probe: geometry.timeline_probe,
        }
    }
}

fn source_resolution<'a>(input: &ResolveInput<'a>, observed_at_ms: u64) -> SourceResolution<'a> {
    let continuation = input.evidence.continuation_source.as_deref();
    let source = selected_source(input.entry, &input.evidence.origins, continuation);
    let compatible = source_compatible(source, continuation);
    let assessment = source
        .map(|source| input.entry.evidence_assessment_for(source, observed_at_ms))
        .unwrap_or_default();
    let total = effective_total(&assessment, input.evidence.stored_total, compatible);
    let present = if compatible {
        input.evidence.present.clone()
    } else {
        Vec::new()
    };
    let layout = layout(
        &assessment,
        source,
        &input.evidence.independent_object_sources,
    );
    SourceResolution {
        source,
        assessment,
        total,
        present,
        layout,
    }
}

fn media_geometry(input: &ResolveInput<'_>, source: &SourceResolution<'_>) -> MediaGeometry {
    let bitrate = input
        .catalog
        .estimated_bitrate_for_total(&input.evidence.post, source.total, input.params)
        .max(1);
    let duration = duration_ms(input.entry, source.total, bitrate);
    let startup = startup::footprint(startup::Inputs {
        entry: input.entry,
        layout: source.layout,
        total: source.total,
        duration_ms: duration,
        present: &source.present,
    });
    let playable_ranges = playable::ranges(
        source.layout,
        playable::Inputs {
            entry: input.entry,
            total: source.total,
            duration_ms: duration,
            chunk_bytes: input.params.chunk_bytes,
            present: &source.present,
        },
    );
    MediaGeometry {
        bitrate,
        duration,
        startup,
        timeline_probe: timeline_probe(input.entry, source.layout, &playable_ranges),
        playable_ranges,
    }
}
