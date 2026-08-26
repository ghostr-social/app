use super::super::{allocation::REQUEST_SLICE_BYTES, PlayableRange};
use crate::adaptive::MediaLayout;
use crate::catalog::CatalogEntry;
use crate::ByteRange;

#[derive(Clone, Copy)]
pub(super) struct Inputs<'a> {
    pub(super) entry: &'a CatalogEntry,
    pub(super) total: Option<u64>,
    pub(super) duration_ms: u64,
    pub(super) chunk_bytes: u64,
    pub(super) present: &'a [ByteRange],
}

pub(super) fn ranges(layout: MediaLayout, inputs: Inputs<'_>) -> Vec<PlayableRange> {
    let Some(total) = inputs.total else {
        return vec![bootstrap_range(&inputs)];
    };
    match layout {
        MediaLayout::Unknown => vec![bootstrap_range(&inputs)],
        MediaLayout::RequiresCompleteFile => vec![complete_range(total, inputs.duration_ms)],
        MediaLayout::Streamable => streamable_ranges(total, &inputs),
    }
}

fn complete_range(total: u64, duration_ms: u64) -> PlayableRange {
    PlayableRange {
        bytes: ByteRange::new(0, total),
        playable_ms: duration_ms,
    }
}

fn streamable_ranges(total: u64, inputs: &Inputs<'_>) -> Vec<PlayableRange> {
    if let Some(timeline) = inputs.entry.timeline() {
        return timeline
            .playable_extents()
            .into_iter()
            .map(|extent| PlayableRange {
                bytes: extent.bytes,
                playable_ms: extent.playable_ms,
            })
            .collect();
    }
    split(total, inputs.chunk_bytes.max(1))
        .into_iter()
        .map(|bytes| PlayableRange {
            playable_ms: range_duration(bytes, total, inputs.duration_ms),
            bytes,
        })
        .collect()
}

fn bootstrap_range(inputs: &Inputs<'_>) -> PlayableRange {
    let bound = inputs.chunk_bytes.clamp(1, REQUEST_SLICE_BYTES);
    let start = bootstrap_start(inputs.present).min(inputs.total.unwrap_or(u64::MAX));
    let next_end = start.saturating_add(bound);
    let end = inputs.total.map_or(next_end, |known| known.min(next_end));
    let bytes = if start < end {
        ByteRange::new(start, end)
    } else {
        ByteRange::new(0, end)
    };
    PlayableRange {
        bytes,
        playable_ms: inputs
            .total
            .map_or(1, |known| range_duration(bytes, known, inputs.duration_ms)),
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
