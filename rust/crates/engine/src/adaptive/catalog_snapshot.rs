use super::{
    CandidateSnapshot, FeedOffset, InFlightRange, MediaLayout, OriginHealth, PlayableRange,
    ViewProbability,
};
use crate::catalog::{Catalog, CatalogEntry};
use crate::{ByteRange, EngineParams, PostId};

pub struct CandidateEvidence {
    pub post: PostId,
    pub feed_offset: FeedOffset,
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
    let total = entry.total_bytes().filter(|total| *total > 0);
    let bitrate = catalog.estimated_bitrate(&evidence.post, params).max(1);
    let duration = duration_ms(entry, total, bitrate);
    let layout = layout(entry);
    let playable_ranges = playable_ranges(
        layout,
        PlayableInputs {
            entry,
            total,
            duration_ms: duration,
            chunk_bytes: params.chunk_bytes,
            present: &evidence.present,
        },
    );
    Some(CandidateSnapshot {
        post: evidence.post,
        feed_offset: evidence.feed_offset,
        view_probability: evidence.view_probability,
        total_bytes: total,
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

struct PlayableInputs<'a> {
    entry: &'a CatalogEntry,
    total: Option<u64>,
    duration_ms: u64,
    chunk_bytes: u64,
    present: &'a [ByteRange],
}

fn playable_ranges(layout: MediaLayout, inputs: PlayableInputs<'_>) -> Vec<PlayableRange> {
    let PlayableInputs {
        entry,
        total,
        duration_ms,
        chunk_bytes,
        present,
    } = inputs;
    let Some(total) = total else {
        return vec![bootstrap_range(None, duration_ms, chunk_bytes, present)];
    };
    if layout == MediaLayout::Unknown {
        return vec![bootstrap_range(
            Some(total),
            duration_ms,
            chunk_bytes,
            present,
        )];
    }
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

fn bootstrap_range(
    total: Option<u64>,
    duration_ms: u64,
    chunk_bytes: u64,
    present: &[ByteRange],
) -> PlayableRange {
    let bound = chunk_bytes.clamp(1, super::allocation::REQUEST_SLICE_BYTES);
    let start = bootstrap_start(present).min(total.unwrap_or(u64::MAX));
    let next_end = start.saturating_add(bound);
    let end = total.map_or(next_end, |known| known.min(next_end));
    let bytes = match start < end {
        true => ByteRange::new(start, end),
        false => ByteRange::new(0, end),
    };
    PlayableRange {
        bytes,
        playable_ms: total.map_or(1, |known| range_duration(bytes, known, duration_ms)),
    }
}

fn bootstrap_start(present: &[ByteRange]) -> u64 {
    crate::media_timeline::normalize(present.to_vec())
        .first()
        .filter(|range| range.start == 0)
        .map_or(0, |range| range.end)
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
