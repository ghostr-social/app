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
    is_startable_with(
        StartabilityInputs {
            catalog,
            params,
            head_seconds: params.head_seconds,
        },
        post,
        have,
    )
}

struct StartabilityInputs<'a> {
    catalog: &'a Catalog,
    params: &'a EngineParams,
    head_seconds: u64,
}

fn is_startable_with(inputs: StartabilityInputs<'_>, post: &PostId, have: &[ByteRange]) -> bool {
    let Some(entry) = inputs.catalog.lookup(post) else {
        return false;
    };
    if let Some(startable) = super::timeline::is_startable(entry, have, inputs.head_seconds) {
        return startable;
    }
    let input = PlanInput {
        size_bytes: entry.total_bytes(),
        bitrate_bps: inputs.catalog.estimated_bitrate(post, inputs.params),
        needs_tail_probe: entry.needs_tail_probe(),
    };
    let plan = ChunkPlan::from_input_with_head_seconds(input, inputs.params, inputs.head_seconds);
    head_complete(&plan, have) && moov_present(&plan, have)
}

pub(super) struct InventoryInputs<'a> {
    pub(super) catalog: &'a Catalog,
    pub(super) focus: &'a FocusState,
    pub(super) present: &'a PresentRanges,
    pub(super) params: &'a EngineParams,
    pub(super) head_seconds: &'a dyn Fn(&PostId) -> u64,
}

pub(super) fn count_inventory(inputs: InventoryInputs<'_>) -> InventoryCounts {
    let upcoming = upcoming_window(inputs.focus, inputs.params.startable_window);
    let startable = upcoming
        .iter()
        .take_while(|post| {
            is_startable_with(
                StartabilityInputs {
                    catalog: inputs.catalog,
                    params: inputs.params,
                    head_seconds: (inputs.head_seconds)(post),
                },
                post,
                inputs.present.ranges(post),
            )
        })
        .count();
    InventoryCounts {
        considered: upcoming.len(),
        startable,
        target: inputs.params.startable_target.min(upcoming.len()),
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
