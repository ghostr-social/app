//! One planning pass: on-disk byte ranges plus engine state in, the
//! ordered chunk-request list and per-post source URLs out.

use crate::engine::host_stats::{host_of, HostStats};
use crate::engine::inventory_controller::{Mode, PresentRanges};
use crate::engine::scoring::{next_work, ChunkRequest, NextWorkContext};
use crate::engine::tiers::DemandSignals;
use crate::engine::{ByteRange, PostId};
use crate::video::delivery_retry::RetryBook;
use crate::video::delivery_state::DeliveryState;
use std::collections::HashMap;

/// Everything a planning pass reads besides the engine state.
pub(crate) struct PlanInputs<'a> {
    pub stats: &'a HostStats,
    pub retry: &'a RetryBook,
    pub present: &'a HashMap<PostId, Vec<ByteRange>>,
    pub demand: DemandSignals,
}

pub(crate) struct PlannedWork {
    pub transfers: Vec<PlannedTransfer>,
}

#[derive(Clone)]
pub(crate) struct PlannedTransfer {
    pub request: ChunkRequest,
    pub url: String,
}

/// Runs the pure engine planner over the manager's current picture.
/// Posts with no source left to try drop out entirely: they are
/// terminal, not work to reschedule on the next pass.
pub(crate) fn planned_work(state: &mut DeliveryState, inputs: PlanInputs<'_>) -> PlannedWork {
    let inventory = state.observe_inventory(&as_present_ranges(inputs.present));
    let (urls, factors) = source_choices(state, &inputs, inventory.mode);
    let present_of = |post: &PostId| inputs.present.get(post).cloned().unwrap_or_default();
    let factor_of = |post: &PostId| factors.get(post).copied().unwrap_or(1.0);
    let requests = next_work(&NextWorkContext {
        catalog: state.catalog(),
        focus: state.focus(),
        params: state.params(),
        inventory,
        demand: inputs.demand,
        present: &present_of,
        host_factor: &factor_of,
    });
    let transfers = pair_requests(requests, &urls);
    PlannedWork { transfers }
}

fn pair_requests(
    requests: Vec<ChunkRequest>,
    urls: &HashMap<PostId, String>,
) -> Vec<PlannedTransfer> {
    requests
        .into_iter()
        .filter_map(|request| {
            let url = urls.get(&request.chunk.post)?.clone();
            Some(PlannedTransfer { request, url })
        })
        .collect()
}

fn as_present_ranges(present: &HashMap<PostId, Vec<ByteRange>>) -> PresentRanges {
    let mut ranges = PresentRanges::new();
    for (post, have) in present {
        ranges.set(post.clone(), have.clone());
    }
    ranges
}

/// Best source URL and host speed factor per catalogued window post.
/// Sources the retry policy retired are not candidates at all, so a
/// post falls back to its healthy mirrors on its own.
fn source_choices(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    mode: Mode,
) -> (HashMap<PostId, String>, HashMap<PostId, f64>) {
    let mut urls = HashMap::new();
    let mut factors = HashMap::new();
    for post in state.window_posts() {
        let Some(entry) = state.catalog().lookup(&post) else {
            continue;
        };
        if entry.total_bytes().is_none() {
            continue;
        }
        let live = inputs.retry.live_urls(&post, &entry.meta.urls);
        let Some(url) = inputs.stats.best_source(&live, mode).into_iter().next() else {
            continue;
        };
        let factor = host_of(&url)
            .map(|host| inputs.stats.host_factor(&host, mode))
            .unwrap_or(1.0);
        urls.insert(post.clone(), url);
        factors.insert(post, factor);
    }
    (urls, factors)
}
