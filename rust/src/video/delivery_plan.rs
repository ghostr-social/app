//! One planning pass: on-disk byte ranges plus engine state in, the
//! ordered chunk-request list and per-post source URLs out.

use crate::engine::host_stats::{host_of, HostStats};
use crate::engine::inventory_controller::{Mode, PresentRanges};
use crate::engine::scoring::{next_work, ChunkRequest, NextWorkContext};
use crate::engine::tiers::DemandSignals;
use crate::engine::{ByteRange, PostId};
use crate::video::delivery_state::DeliveryState;
use std::collections::HashMap;

pub(crate) struct PlannedWork {
    pub requests: Vec<ChunkRequest>,
    pub urls: HashMap<PostId, String>,
}

/// Runs the pure engine planner over the manager's current picture.
pub(crate) fn planned_work(
    state: &mut DeliveryState,
    stats: &HostStats,
    present: &HashMap<PostId, Vec<ByteRange>>,
    demand: DemandSignals,
) -> PlannedWork {
    let inventory = state.observe_inventory(&as_present_ranges(present));
    let (urls, factors) = source_choices(state, stats, inventory.mode);
    let present_of = |post: &PostId| present.get(post).cloned().unwrap_or_default();
    let factor_of = |post: &PostId| factors.get(post).copied().unwrap_or(1.0);
    let requests = next_work(&NextWorkContext {
        catalog: state.catalog(),
        focus: state.focus(),
        params: state.params(),
        inventory,
        demand,
        present: &present_of,
        host_factor: &factor_of,
    });
    PlannedWork { requests, urls }
}

fn as_present_ranges(present: &HashMap<PostId, Vec<ByteRange>>) -> PresentRanges {
    let mut ranges = PresentRanges::new();
    for (post, have) in present {
        ranges.set(post.clone(), have.clone());
    }
    ranges
}

/// Best source URL and host speed factor per catalogued window post.
fn source_choices(
    state: &DeliveryState,
    stats: &HostStats,
    mode: Mode,
) -> (HashMap<PostId, String>, HashMap<PostId, f64>) {
    let mut urls = HashMap::new();
    let mut factors = HashMap::new();
    for post in state.window_posts() {
        let Some(entry) = state.catalog().lookup(&post) else {
            continue;
        };
        let Some(url) = stats.best_source(&entry.meta.urls, mode).into_iter().next() else {
            continue;
        };
        let factor = host_of(&url)
            .map(|host| stats.host_factor(&host, mode))
            .unwrap_or(1.0);
        urls.insert(post.clone(), url);
        factors.insert(post, factor);
    }
    (urls, factors)
}
