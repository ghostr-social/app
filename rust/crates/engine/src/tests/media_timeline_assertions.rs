use crate::media_timeline::{normalize, MediaTimeline};
use crate::ByteRange;

pub(super) fn duration_ms(timeline: &MediaTimeline) -> u64 {
    timeline
        .media
        .iter()
        .map(|range| range.end_ms)
        .max()
        .unwrap_or(0)
}

pub(super) fn media_ranges(timeline: &MediaTimeline, start_ms: u64, end_ms: u64) -> Vec<ByteRange> {
    assert!(start_ms < end_ms);
    normalize(
        timeline
            .media
            .iter()
            .filter(|range| range.start_ms < end_ms && range.end_ms > start_ms)
            .map(|range| range.bytes)
            .collect(),
    )
}

pub(super) fn required_ranges(
    timeline: &MediaTimeline,
    start_ms: u64,
    end_ms: u64,
) -> Vec<ByteRange> {
    let mut ranges = timeline.metadata.clone();
    ranges.extend(media_ranges(timeline, start_ms, end_ms));
    normalize(ranges)
}
