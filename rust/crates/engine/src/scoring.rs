//! Within-tier ordering and the pure planning entry point: turns the
//! catalog, focus window, and on-disk byte ranges into an ordered list
//! of chunk requests for the transfer layer. No IO, no async.

use crate::catalog::Catalog;
use crate::focus::FocusState;
use crate::inventory_controller::InventoryState;
use crate::playback::PLAYBACK_SLICE_BYTES;
use crate::tiers::{classify, DemandSignals, PostInventory, Tier};
use crate::{ByteRange, ChunkId, EngineParams, PostId};
use std::cmp::Ordering;
use std::collections::HashSet;

mod fallback;
mod frontier;
mod timeline;

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
    pub startup_depth_bytes: u64,
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
pub fn compare(a: &ChunkRequest, b: &ChunkRequest) -> Ordering {
    a.tier
        .cmp(&b.tier)
        .then_with(|| startup_depth_order(a, b))
        .then_with(|| b.score.total_cmp(&a.score))
        .then_with(|| a.chunk.post.cmp(&b.chunk.post))
        .then_with(|| a.chunk.range.start.cmp(&b.chunk.range.start))
}

fn startup_depth_order(a: &ChunkRequest, b: &ChunkRequest) -> Ordering {
    match a.tier == Tier::T2Startability && b.tier == Tier::T2Startability {
        true => a.startup_depth_bytes.cmp(&b.startup_depth_bytes),
        false => Ordering::Equal,
    }
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
    pub head_seconds: &'a dyn Fn(&PostId) -> u64,
    pub tail_end: &'a dyn Fn(&PostId) -> Option<u64>,
    pub media_window: &'a dyn Fn(&PostId) -> Option<crate::media_timeline::PlaybackWindow>,
    pub direct_range: &'a dyn Fn(&PostId) -> Option<ByteRange>,
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
    let have = (ctx.present)(post);
    let (heads, tails) = match timeline::planned_ranges(ctx, post, &have) {
        Some(ranges) => ranges,
        None => fallback::planned_ranges(ctx, post, &have),
    };
    let has_heads = !heads.is_empty();
    let inventory = PostInventory::new(ctx.inventory, heads.is_empty());
    let demand = demand_for(ctx, post);
    let Some(tier) = classify(post, ctx.focus, inventory, demand) else {
        return Vec::new();
    };
    let is_current = ctx.focus.current() == Some(post);
    let ranges = match (has_heads, is_current) {
        (true, true) => heads,
        (true, false) => split_startup_ranges(heads),
        (false, _) => tails,
    };
    score_requests(ctx, post, tier, ranges)
}

fn split_startup_ranges(ranges: Vec<ByteRange>) -> Vec<ByteRange> {
    ranges.into_iter().flat_map(split_startup_range).collect()
}

fn split_startup_range(range: ByteRange) -> Vec<ByteRange> {
    let mut start = range.start;
    let mut split = Vec::new();
    while start < range.end {
        let end = start.saturating_add(PLAYBACK_SLICE_BYTES).min(range.end);
        split.push(ByteRange::new(start, end));
        start = end;
    }
    split
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
    let mut assumed = (ctx.present)(post);
    ranges
        .into_iter()
        .map(|range| {
            let startup_depth_bytes = contiguous_prefix_end(&assumed);
            let request = ChunkRequest {
                chunk: ChunkId {
                    post: post.clone(),
                    range,
                },
                tier,
                score,
                startup_depth_bytes,
            };
            assumed.push(range);
            request
        })
        .collect()
}

fn contiguous_prefix_end(have: &[ByteRange]) -> u64 {
    let mut end = 0;
    for range in crate::media_timeline::normalize(have.to_vec()) {
        if range.start > end {
            break;
        }
        end = end.max(range.end);
    }
    end
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
