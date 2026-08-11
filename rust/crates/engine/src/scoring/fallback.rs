use super::frontier::bounded_tails;
use super::NextWorkContext;
use crate::chunk_plan::{ChunkPlan, PlanInput};
use crate::{ByteRange, PostId};

pub(super) fn planned_ranges(
    ctx: &NextWorkContext<'_>,
    post: &PostId,
    have: &[ByteRange],
) -> (Vec<ByteRange>, Vec<ByteRange>) {
    let Some(plan) = plan_for(ctx, post) else {
        return (Vec::new(), Vec::new());
    };
    let (mut heads, tails): (Vec<_>, Vec<_>) = missing_chunks(&plan, have)
        .into_iter()
        .partition(|chunk| chunk.start < plan.head_bytes());
    let probe = plan.next_missing_tail_probe(have);
    let entry = ctx.catalog.lookup(post).expect("plan requires entry");
    if entry.needs_tail_probe() {
        heads.extend(probe);
    }
    let tails = match (entry.needs_timeline_probe(), target_met(ctx), probe) {
        (true, true, Some(range)) if !entry.needs_tail_probe() => vec![range],
        _ => bounded_tails(ctx, post, plan.head_bytes(), tails),
    };
    (heads, tails)
}

fn plan_for(ctx: &NextWorkContext<'_>, post: &PostId) -> Option<ChunkPlan> {
    let entry = ctx.catalog.lookup(post)?;
    let input = PlanInput {
        size_bytes: entry.total_bytes(),
        bitrate_bps: ctx.catalog.estimated_bitrate(post, ctx.params),
        needs_tail_probe: entry.needs_timeline_probe(),
    };
    Some(ChunkPlan::from_input_with_head_seconds(
        input,
        ctx.params,
        (ctx.head_seconds)(post),
    ))
}

fn missing_chunks(plan: &ChunkPlan, have: &[ByteRange]) -> Vec<ByteRange> {
    let mut assumed = have.to_vec();
    let mut missing = Vec::new();
    while let Some(chunk) = plan.next_missing_chunk(&assumed) {
        assumed.push(chunk);
        missing.push(chunk);
    }
    missing
}

fn target_met(ctx: &NextWorkContext<'_>) -> bool {
    ctx.inventory.counts.startable >= ctx.inventory.counts.target
}
