use super::NextWorkContext;
use crate::{ByteRange, PostId};

pub(super) fn bounded_tails(
    ctx: &NextWorkContext<'_>,
    post: &PostId,
    head_end: u64,
    tails: Vec<ByteRange>,
) -> Vec<ByteRange> {
    let end = match ctx.focus.current() == Some(post) {
        true => current_end(ctx, post, head_end, &tails),
        false => head_end.saturating_add(ctx.params.chunk_bytes.max(1)),
    };
    tails
        .into_iter()
        .filter(|range| range.start < end)
        .collect()
}

fn current_end(
    ctx: &NextWorkContext<'_>,
    post: &PostId,
    head_end: u64,
    tails: &[ByteRange],
) -> u64 {
    if let Some(end) = (ctx.tail_end)(post) {
        return end;
    }
    if ctx.demand.gateway_demand || ctx.demand.buffer_below_emergency {
        return tails.first().map_or(head_end, |range| range.end);
    }
    head_end.saturating_add(ctx.params.chunk_bytes.max(1))
}
