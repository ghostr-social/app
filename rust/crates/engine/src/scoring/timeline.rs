use super::NextWorkContext;
use crate::media_timeline::{normalize, MediaTimeline, PlaybackWindow};
use crate::{ByteRange, PostId};

pub(super) fn planned_ranges(
    ctx: &NextWorkContext<'_>,
    post: &PostId,
    have: &[ByteRange],
) -> Option<(Vec<ByteRange>, Vec<ByteRange>)> {
    let timeline = ctx.catalog.lookup(post)?.timeline()?;
    let startup = startup_window(ctx, post);
    let heads = missing_exact(
        timeline.required_ranges(startup),
        have,
        ctx.params.chunk_bytes,
    );
    if !heads.is_empty() {
        return Some((heads, Vec::new()));
    }
    let desired = desired_window(ctx, post, timeline, startup);
    let mut ranges = timeline.media_ranges(desired);
    ranges.extend((ctx.direct_range)(post));
    let tails = missing_exact(normalize(ranges), have, ctx.params.chunk_bytes);
    Some((Vec::new(), tails))
}

fn startup_window(ctx: &NextWorkContext<'_>, post: &PostId) -> PlaybackWindow {
    let end = (ctx.head_seconds)(post).saturating_mul(1_000).max(1);
    PlaybackWindow::try_new(0, end).expect("positive startup horizon")
}

fn desired_window(
    ctx: &NextWorkContext<'_>,
    post: &PostId,
    timeline: &MediaTimeline,
    startup: PlaybackWindow,
) -> PlaybackWindow {
    if let Some(window) = (ctx.media_window)(post) {
        return window;
    }
    let start = startup
        .end_ms()
        .min(timeline.duration_ms().saturating_sub(1));
    let end = start
        .saturating_add(1_000)
        .min(timeline.duration_ms())
        .max(start + 1);
    PlaybackWindow::try_new(start, end).expect("bounded reserve horizon")
}

fn missing_exact(wanted: Vec<ByteRange>, have: &[ByteRange], chunk: u64) -> Vec<ByteRange> {
    let mut missing = Vec::new();
    for range in wanted {
        append_missing(range, have, chunk.max(1), &mut missing);
    }
    missing
}

fn append_missing(range: ByteRange, have: &[ByteRange], chunk: u64, output: &mut Vec<ByteRange>) {
    let mut cursor = range.start;
    while cursor < range.end {
        if let Some(reach) = furthest_reach(cursor, have) {
            cursor = reach.min(range.end);
            continue;
        }
        let gap_end = next_present(cursor, range.end, have);
        append_split(cursor, gap_end, chunk, output);
        cursor = gap_end;
    }
}

fn append_split(mut start: u64, end: u64, chunk: u64, output: &mut Vec<ByteRange>) {
    while start < end {
        let next = start.saturating_add(chunk).min(end);
        output.push(ByteRange::new(start, next));
        start = next;
    }
}

fn furthest_reach(cursor: u64, have: &[ByteRange]) -> Option<u64> {
    have.iter()
        .filter(|range| range.start <= cursor && range.end > cursor)
        .map(|range| range.end)
        .max()
}

fn next_present(cursor: u64, end: u64, have: &[ByteRange]) -> u64 {
    have.iter()
        .filter(|range| range.start > cursor)
        .map(|range| range.start)
        .min()
        .unwrap_or(end)
        .min(end)
}
