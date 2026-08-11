use super::{InventoryCounts, PresentRanges};
use crate::catalog::Catalog;
use crate::chunk_plan::{ChunkPlan, PlanInput};
use crate::focus::FocusState;
use crate::{ByteRange, EngineParams, PostId};

/// Whether a post's startup prefix and container metadata are present.
pub fn is_startable(
    catalog: &Catalog,
    post: &PostId,
    have: &[ByteRange],
    params: &EngineParams,
) -> bool {
    is_startable_with(catalog, post, have, params, params.head_seconds)
}

pub(crate) fn is_startable_with(
    catalog: &Catalog,
    post: &PostId,
    have: &[ByteRange],
    params: &EngineParams,
    head_seconds: u64,
) -> bool {
    let Some(entry) = catalog.lookup(post) else {
        return false;
    };
    let input = PlanInput {
        size_bytes: entry.total_bytes(),
        duration_ms: entry.meta.duration_ms,
        bitrate_bps: catalog.estimated_bitrate(post, params),
    };
    let plan = ChunkPlan::from_input_with_head_seconds(input, params, head_seconds);
    head_complete(&plan, have) && moov_present(&plan, have)
}

pub(super) fn count_inventory(
    catalog: &Catalog,
    focus: &FocusState,
    present: &PresentRanges,
    params: &EngineParams,
    head_seconds: &dyn Fn(&PostId) -> u64,
) -> InventoryCounts {
    let upcoming = upcoming_window(focus, params.startable_window);
    let startable = upcoming
        .iter()
        .filter(|post| {
            is_startable_with(
                catalog,
                post,
                present.ranges(post),
                params,
                head_seconds(post),
            )
        })
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
    plan.tail_probe_range()
        .is_some_and(|range| covers(range, have))
}

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
