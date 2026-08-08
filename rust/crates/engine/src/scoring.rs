//! Within-tier ordering and the pure planning entry point: turns the
//! catalog, focus window, and on-disk byte ranges into an ordered list
//! of chunk requests for the transfer layer. No IO, no async.

use crate::catalog::Catalog;
use crate::chunk_plan::{ChunkPlan, PlanInput};
use crate::focus::FocusState;
use crate::inventory_controller::{is_startable, InventoryState};
use crate::tiers::{classify, DemandSignals, PostInventory, Tier};
use crate::{ByteRange, ChunkId, EngineParams, PostId};
use std::cmp::Ordering;
use std::collections::HashSet;

/// Per-step weight decay for posts ahead of the viewer.
const AHEAD_DECAY: f64 = 0.7;
/// Extra discount for posts behind the viewer (scroll-back).
const BEHIND_DISCOUNT: f64 = 0.15;

/// One ordered unit of transfer work granted to the downloader.
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkRequest {
    pub chunk: ChunkId,
    pub tier: Tier,
    pub score: f64,
}

/// Exponential decay by scroll distance; behind items take a heavy
/// extra discount on top of their distance decay.
pub(crate) fn position_weight(distance: i64) -> f64 {
    let decay = AHEAD_DECAY.powi(distance.unsigned_abs().min(64) as i32);
    match distance >= 0 {
        true => decay,
        false => BEHIND_DISCOUNT * decay,
    }
}

/// Milestone playback seconds a chunk unlocks, per byte fetched.
pub(crate) fn value_per_byte(milestone_seconds: f64, chunk_bytes: u64) -> f64 {
    milestone_seconds / chunk_bytes.max(1) as f64
}

/// Total deterministic ordering: tier first, higher score next, then
/// post id and range start so equal scores cannot reorder runs.
pub(crate) fn compare(a: &ChunkRequest, b: &ChunkRequest) -> Ordering {
    a.tier
        .cmp(&b.tier)
        .then_with(|| b.score.total_cmp(&a.score))
        .then_with(|| a.chunk.post.cmp(&b.chunk.post))
        .then_with(|| a.chunk.range.start.cmp(&b.chunk.range.start))
}

/// Everything `next_work` reads. `present` reports the byte ranges
/// already on disk; `host_factor` is the caller-supplied host speed
/// multiplier so host statistics stay outside this module.
pub struct NextWorkContext<'a> {
    pub catalog: &'a Catalog,
    pub focus: &'a FocusState,
    pub params: &'a EngineParams,
    pub inventory: InventoryState,
    pub demand: DemandSignals,
    pub present: &'a dyn Fn(&PostId) -> Vec<ByteRange>,
    pub host_factor: &'a dyn Fn(&PostId) -> f64,
}

/// The single pure planning entry point: every chunk the engine wants
/// right now, most urgent first. Posts outside the focus window are
/// never emitted, so scroll-past cancellation falls out of the order.
pub fn next_work(ctx: &NextWorkContext<'_>) -> Vec<ChunkRequest> {
    let mut seen = HashSet::new();
    let mut requests = Vec::new();
    for post in ctx.focus.window() {
        if seen.insert(post.clone()) {
            requests.extend(post_requests(ctx, post));
        }
    }
    requests.sort_by(compare);
    requests
}

/// Missing startability work (head chunks, then the moov probe) is
/// emitted first; tail chunks are withheld until the post is startable.
fn post_requests(ctx: &NextWorkContext<'_>, post: &PostId) -> Vec<ChunkRequest> {
    let Some(plan) = plan_for(ctx, post) else {
        return Vec::new();
    };
    let have = (ctx.present)(post);
    let (mut heads, tails): (Vec<_>, Vec<_>) = missing_chunks(&plan, &have)
        .into_iter()
        .partition(|chunk| chunk.start < plan.head_bytes());
    heads.extend(missing_probe(ctx, post, &plan, &have));
    let inventory =
        PostInventory::new(ctx.inventory, ctx.params.startable_window, heads.is_empty());
    let Some(tier) = classify(post, ctx.focus, inventory, demand_for(ctx, post)) else {
        return Vec::new();
    };
    let ranges = match heads.is_empty() {
        true => tails,
        false => heads,
    };
    score_requests(ctx, post, tier, ranges)
}

fn plan_for(ctx: &NextWorkContext<'_>, post: &PostId) -> Option<ChunkPlan> {
    let entry = ctx.catalog.lookup(post)?;
    let input = PlanInput {
        size_bytes: entry.total_bytes(),
        duration_ms: entry.meta.duration_ms,
        bitrate_bps: ctx.catalog.estimated_bitrate(post, ctx.params),
    };
    Some(ChunkPlan::from_input(input, ctx.params))
}

/// All planned chunks not yet covered, in head-then-tail fetch order.
fn missing_chunks(plan: &ChunkPlan, have: &[ByteRange]) -> Vec<ByteRange> {
    let mut assumed = have.to_vec();
    let mut missing = Vec::new();
    while let Some(chunk) = plan.next_missing_chunk(&assumed) {
        assumed.push(chunk);
        missing.push(chunk);
    }
    missing
}

/// With the whole head assumed on disk, `is_startable` reduces to "is
/// the moov question settled"; when it is not, the probe range is the
/// missing piece (`None` while the file size is still unknown).
fn missing_probe(
    ctx: &NextWorkContext<'_>,
    post: &PostId,
    plan: &ChunkPlan,
    have: &[ByteRange],
) -> Option<ByteRange> {
    let mut assumed = have.to_vec();
    assumed.extend(plan.head_ranges());
    match is_startable(ctx.catalog, post, &assumed, ctx.params) {
        true => None,
        false => plan.tail_probe_range(),
    }
}

/// Demand belongs to the playing post; other posts see calm signals.
/// Commitment also follows from the tracked watch time.
fn demand_for(ctx: &NextWorkContext<'_>, post: &PostId) -> DemandSignals {
    match ctx.focus.current() == Some(post) {
        true => DemandSignals {
            viewer_committed: ctx.demand.viewer_committed
                || ctx.focus.is_committed(ctx.params.commitment_ms),
            ..ctx.demand
        },
        false => DemandSignals::default(),
    }
}

fn score_requests(
    ctx: &NextWorkContext<'_>,
    post: &PostId,
    tier: Tier,
    ranges: Vec<ByteRange>,
) -> Vec<ChunkRequest> {
    let weight = position_weight(ctx.focus.distance_of(post).unwrap_or(0));
    let score = weight * post_value(ctx, post) * (ctx.host_factor)(post);
    ranges
        .into_iter()
        .map(|range| ChunkRequest {
            chunk: ChunkId {
                post: post.clone(),
                range,
            },
            tier,
            score,
        })
        .collect()
}

/// Seconds gained per byte at the post's bitrate, measured on one
/// standard chunk so every chunk of a post scores identically and the
/// range-start tie-break keeps them in file order.
fn post_value(ctx: &NextWorkContext<'_>, post: &PostId) -> f64 {
    let bitrate = ctx.catalog.estimated_bitrate(post, ctx.params);
    let chunk_bytes = ctx.params.chunk_bytes.max(1);
    let chunk_seconds = chunk_bytes.saturating_mul(8) as f64 / bitrate.max(1) as f64;
    value_per_byte(chunk_seconds, chunk_bytes)
}
