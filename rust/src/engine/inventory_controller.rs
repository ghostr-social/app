//! Inventory control loop (plan §3): counts how many upcoming posts
//! are startable and switches between hunger and comfort with
//! hysteresis. Pure bookkeeping — no IO, no clocks.

use crate::engine::catalog::Catalog;
use crate::engine::chunk_plan::{ChunkPlan, PlanInput};
use crate::engine::focus::FocusState;
use crate::engine::{ByteRange, EngineParams, PostId};
use std::collections::HashMap;

/// Control-loop mode: hunger races for startability, comfort deepens.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mode {
    Hunger,
    Comfort,
}

/// Byte ranges already on disk, per post. Posts never set are empty.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PresentRanges {
    by_post: HashMap<PostId, Vec<ByteRange>>,
}

impl PresentRanges {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, post: PostId, ranges: Vec<ByteRange>) {
        self.by_post.insert(post, ranges);
    }

    pub fn ranges(&self, post: &PostId) -> &[ByteRange] {
        self.by_post.get(post).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Startability inventory over the upcoming window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InventoryCounts {
    /// Posts actually considered: current + ahead, capped at
    /// `startable_window`.
    pub considered: usize,
    /// How many of those are startable right now.
    pub startable: usize,
    /// Effective target: the configured target, never above
    /// `considered`.
    pub target: usize,
}

/// One control-loop observation: the counts and the resulting mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InventoryState {
    pub counts: InventoryCounts,
    pub mode: Mode,
}

/// Remembers the mode across observations so hysteresis has memory.
/// A fresh controller starts hungry: nothing is fetched yet.
#[derive(Clone, Debug)]
pub struct InventoryController {
    params: EngineParams,
    mode: Mode,
}

impl InventoryController {
    pub fn new(params: EngineParams) -> Self {
        Self {
            params,
            mode: Mode::Hunger,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Recounts startable posts in the upcoming window and applies the
    /// hysteresis rule. The returned mode is remembered for the next
    /// observation.
    pub fn inventory_state(
        &mut self,
        catalog: &Catalog,
        focus: &FocusState,
        present: &PresentRanges,
    ) -> InventoryState {
        let counts = count_inventory(catalog, focus, present, &self.params);
        self.mode = next_mode(self.mode, counts.startable, counts.target);
        InventoryState {
            counts,
            mode: self.mode,
        }
    }
}

/// Hysteresis rule (plan §3): enter comfort once the target is met;
/// fall back to hunger only below `target - 1`, so losing a single
/// startable post cannot flap the mode.
pub fn next_mode(current: Mode, startable: usize, target: usize) -> Mode {
    if startable >= target {
        Mode::Comfort
    } else if startable + 1 < target {
        Mode::Hunger
    } else {
        current
    }
}

/// Plan §3 startability: the whole head is on disk and moov is
/// reachable — either duration was known up front (moov placement
/// resolvable from the head), or the tail probe bytes are on disk.
/// Posts missing from the catalog are never startable.
pub fn is_startable(
    catalog: &Catalog,
    post: &PostId,
    have: &[ByteRange],
    params: &EngineParams,
) -> bool {
    let Some(entry) = catalog.lookup(post) else {
        return false;
    };
    let input = PlanInput {
        size_bytes: entry.total_bytes(),
        duration_ms: entry.meta.duration_ms,
        bitrate_bps: catalog.estimated_bitrate(post, params),
    };
    let plan = ChunkPlan::from_input(input, params);
    head_complete(&plan, have) && moov_present(&plan, have)
}

fn count_inventory(
    catalog: &Catalog,
    focus: &FocusState,
    present: &PresentRanges,
    params: &EngineParams,
) -> InventoryCounts {
    let upcoming = upcoming_window(focus, params.startable_window);
    let startable = upcoming
        .iter()
        .filter(|post| is_startable(catalog, post, present.ranges(post), params))
        .count();
    InventoryCounts {
        considered: upcoming.len(),
        startable,
        target: params.startable_target.min(upcoming.len()),
    }
}

fn upcoming_window(focus: &FocusState, limit: usize) -> &[PostId] {
    let window = focus.window();
    let ahead = &window[focus.current_index().min(window.len())..];
    &ahead[..ahead.len().min(limit)]
}

fn head_complete(plan: &ChunkPlan, have: &[ByteRange]) -> bool {
    plan.head_ranges()
        .into_iter()
        .all(|range| covers(range, have))
}

fn moov_present(plan: &ChunkPlan, have: &[ByteRange]) -> bool {
    if !plan.needs_tail_probe() {
        return true;
    }
    match plan.tail_probe_range() {
        Some(range) => covers(range, have),
        None => false,
    }
}

/// Whether `range` is fully covered by the union of `have` (unsorted,
/// possibly overlapping or adjoining).
fn covers(range: ByteRange, have: &[ByteRange]) -> bool {
    let mut cursor = range.start;
    while cursor < range.end {
        match reach_past(cursor, have) {
            Some(next) => cursor = next,
            None => return false,
        }
    }
    true
}

fn reach_past(cursor: u64, have: &[ByteRange]) -> Option<u64> {
    have.iter()
        .filter(|range| range.start <= cursor && range.end > cursor)
        .map(|range| range.end)
        .max()
}
