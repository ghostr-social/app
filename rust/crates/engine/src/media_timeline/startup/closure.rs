use super::super::TimedRange;
use crate::ByteRange;

pub(super) fn startup_end(media: &[TimedRange]) -> Option<u64> {
    let end = media
        .chunk_by(|left, right| left.track == right.track)
        .map(|track| {
            track
                .iter()
                .map(|sample| sample.time.end_floor_ms())
                .max()
                .unwrap_or(0)
        })
        .min()?
        .min(2_000);
    (end > 0).then_some(end)
}

pub(super) fn ranges(media: &[TimedRange], end_ms: u64) -> Option<Vec<ByteRange>> {
    between(media, 0, end_ms)
}

pub(super) fn between(media: &[TimedRange], start_ms: u64, end_ms: u64) -> Option<Vec<ByteRange>> {
    let mut required = Vec::new();
    for track in media.chunk_by(|left, right| left.track == right.track) {
        let selected: Vec<_> = track
            .iter()
            .enumerate()
            .filter(|(_, sample)| sample.time.intersects_interval(start_ms, end_ms))
            .collect();
        if !covers(&selected, start_ms, end_ms) {
            return None;
        }
        let first = selected.first()?.1.sync_sample? as usize;
        let last = selected.last()?.0;
        required.extend(track.get(first..=last)?.iter().map(|sample| sample.bytes));
    }
    (!required.is_empty()).then_some(required)
}

fn covers(selected: &[(usize, &TimedRange)], start_ms: u64, end_ms: u64) -> bool {
    let Some((_, first)) = selected.first() else {
        return false;
    };
    let timescale = first.time.timescale;
    let mut ordered: Vec<_> = selected.iter().map(|(_, sample)| sample.time).collect();
    ordered.sort_by_key(|time| time.start);
    let mut frontier = i128::from(start_ms) * i128::from(timescale);
    for time in ordered {
        if time.timescale != timescale || i128::from(time.start) * 1_000 > frontier {
            return false;
        }
        frontier = frontier.max(i128::from(time.end) * 1_000);
    }
    frontier >= i128::from(end_ms) * i128::from(timescale)
}
