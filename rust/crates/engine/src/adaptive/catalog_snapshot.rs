use super::{
    CandidateSnapshot, InFlightRange, MediaLayout, OriginHealth, PlayableRange, ViewProbability,
};
use crate::catalog::{Catalog, CatalogEntry};
use crate::{ByteRange, EngineParams, PostId};

pub struct CandidateEvidence {
    pub post: PostId,
    pub feed_distance: usize,
    pub view_probability: ViewProbability,
    pub present: Vec<ByteRange>,
    pub recently_evicted: Vec<ByteRange>,
    pub in_flight: Vec<InFlightRange>,
    pub origins: Vec<OriginHealth>,
}

pub fn candidate_snapshot(
    catalog: &Catalog,
    params: &EngineParams,
    evidence: CandidateEvidence,
) -> Option<CandidateSnapshot> {
    let entry = catalog.lookup(&evidence.post)?;
    let total = entry.total_bytes().filter(|total| *total > 0)?;
    let bitrate = catalog.estimated_bitrate(&evidence.post, params).max(1);
    let duration = duration_ms(entry, total, bitrate);
    let layout = layout(entry);
    let playable_ranges = playable_ranges(entry, layout, total, duration, params.chunk_bytes);
    Some(CandidateSnapshot {
        post: evidence.post,
        feed_distance: evidence.feed_distance,
        view_probability: evidence.view_probability,
        bitrate_bps: bitrate,
        duration_ms: duration,
        layout,
        timeline_probe: timeline_probe(entry, layout, &playable_ranges),
        playable_ranges,
        demanded: None,
        present: evidence.present,
        recently_evicted: evidence.recently_evicted,
        in_flight: evidence.in_flight,
        origins: evidence.origins,
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

fn layout(entry: &CatalogEntry) -> MediaLayout {
    match entry.accepts_byte_ranges() {
        Some(true) => MediaLayout::Streamable,
        Some(false) => MediaLayout::RequiresCompleteFile,
        None => MediaLayout::Unknown,
    }
}

fn duration_ms(entry: &CatalogEntry, total: u64, bitrate: u64) -> u64 {
    entry
        .meta
        .duration_ms
        .filter(|duration| *duration > 0)
        .unwrap_or_else(|| total.saturating_mul(8_000).div_ceil(bitrate))
}

fn playable_ranges(
    entry: &CatalogEntry,
    layout: MediaLayout,
    total: u64,
    duration_ms: u64,
    chunk_bytes: u64,
) -> Vec<PlayableRange> {
    if layout == MediaLayout::RequiresCompleteFile {
        return vec![PlayableRange {
            bytes: ByteRange::new(0, total),
            playable_ms: duration_ms,
        }];
    }
    if let Some(timeline) = entry.timeline() {
        return timeline
            .playable_extents()
            .into_iter()
            .map(|extent| PlayableRange {
                bytes: extent.bytes,
                playable_ms: extent.playable_ms,
            })
            .collect();
    }
    split(total, chunk_bytes.max(1))
        .into_iter()
        .map(|bytes| PlayableRange {
            playable_ms: range_duration(bytes, total, duration_ms),
            bytes,
        })
        .collect()
}

fn split(total: u64, chunk_bytes: u64) -> Vec<ByteRange> {
    let mut ranges = Vec::new();
    let mut start = 0;
    while start < total {
        let end = start.saturating_add(chunk_bytes).min(total);
        ranges.push(ByteRange::new(start, end));
        start = end;
    }
    ranges
}

fn range_duration(range: ByteRange, total: u64, duration_ms: u64) -> u64 {
    let at =
        |offset| u128::from(duration_ms).saturating_mul(u128::from(offset)) / u128::from(total);
    let gain = at(range.end).saturating_sub(at(range.start));
    gain.max(1).min(u128::from(u64::MAX)) as u64
}
