//! One planning pass: on-disk byte ranges plus engine state in, the
//! ordered chunk-request list and per-post source URLs out.

use crate::manager::retry::RetryBook;
use crate::manager::state::DeliveryState;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::inventory_controller::{Mode, PresentRanges};
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scoring::{next_work, ChunkRequest, NextWorkContext};
use ghostr_engine::tiers::DemandSignals;
use ghostr_engine::{ByteRange, PostId};
use playback::playback_plan;
pub(crate) use startup::{startup_seconds, StartupContext};
use std::collections::HashMap;

pub(crate) mod playback;
mod startup;

/// Everything a planning pass reads besides the engine state.
pub(crate) struct PlanInputs<'a> {
    pub stats: &'a HostStats,
    pub retry: &'a RetryBook,
    pub present: &'a HashMap<PostId, Vec<ByteRange>>,
    pub demand: DemandSignals,
    pub observed_at_ms: u64,
    pub demanded_end: Option<u64>,
}

pub(crate) struct PlannedWork {
    pub transfers: Vec<PlannedTransfer>,
    pub emergency: bool,
}

#[derive(Clone)]
pub(crate) struct PlannedTransfer {
    pub request: ChunkRequest,
    pub url: String,
    pub identity: TransferIdentity,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlannedTransferId {
    pub chunk: ghostr_engine::ChunkId,
    pub identity: TransferIdentity,
}

impl PlannedTransfer {
    pub(crate) fn id(&self) -> PlannedTransferId {
        PlannedTransferId {
            chunk: self.request.chunk.clone(),
            identity: self.identity.clone(),
        }
    }
}

/// Runs the pure engine planner over the manager's current picture.
/// Posts with no source left to try drop out entirely: they are
/// terminal, not work to reschedule on the next pass.
pub(crate) fn planned_work(state: &mut DeliveryState, inputs: PlanInputs<'_>) -> PlannedWork {
    let choices = source_choices(state, &inputs);
    let playback = playback_plan(
        state,
        inputs.stats,
        &choices.urls,
        inputs.observed_at_ms,
        inputs.demanded_end,
    );
    let default_head_seconds = state.params().head_seconds;
    let head_seconds = |post: &PostId| {
        choices
            .head_seconds
            .get(post)
            .copied()
            .unwrap_or(default_head_seconds)
    };
    let inventory = state.observe_inventory(&as_present_ranges(inputs.present), &head_seconds);
    let present_of = |post: &PostId| inputs.present.get(post).cloned().unwrap_or_default();
    let factor_of = |post: &PostId| host_factor(&choices.urls, post, inputs.stats, inventory.mode);
    let tail_end = |post: &PostId| playback.tail_end(post);
    let demand = DemandSignals {
        buffer_below_emergency: inputs.demand.buffer_below_emergency || playback.emergency(),
        ..inputs.demand
    };
    let requests = next_work(&NextWorkContext {
        catalog: state.catalog(),
        focus: state.focus(),
        params: state.params(),
        inventory,
        demand,
        present: &present_of,
        host_factor: &factor_of,
        head_seconds: &head_seconds,
        tail_end: &tail_end,
    });
    let transfers = pair_requests(requests, &choices);
    PlannedWork {
        transfers,
        emergency: playback.emergency(),
    }
}

fn pair_requests(requests: Vec<ChunkRequest>, choices: &SourceChoices) -> Vec<PlannedTransfer> {
    requests
        .into_iter()
        .filter_map(|request| {
            let url = choices.urls.get(&request.chunk.post)?.clone();
            let identity = choices.identities.get(&request.chunk.post)?.clone();
            Some(PlannedTransfer {
                request,
                url,
                identity,
            })
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
struct SourceChoices {
    urls: HashMap<PostId, String>,
    identities: HashMap<PostId, TransferIdentity>,
    head_seconds: HashMap<PostId, u64>,
}

fn source_choices(state: &DeliveryState, inputs: &PlanInputs<'_>) -> SourceChoices {
    let mut urls = HashMap::new();
    let mut identities = HashMap::new();
    let mut head_seconds = HashMap::new();
    for post in state.window_posts() {
        let Some(choice) = source_choice(state, inputs, &post) else {
            continue;
        };
        urls.insert(post.clone(), choice.url);
        identities.insert(post.clone(), choice.identity);
        head_seconds.insert(post, choice.head_seconds);
    }
    SourceChoices {
        urls,
        identities,
        head_seconds,
    }
}

struct SourceChoice {
    url: String,
    identity: TransferIdentity,
    head_seconds: u64,
}

fn source_choice(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    post: &PostId,
) -> Option<SourceChoice> {
    let entry = state.catalog().lookup(post)?;
    entry.total_bytes()?;
    let live = inputs.retry.live_urls(post, &entry.meta.urls);
    let url = inputs
        .stats
        .best_source(&live, Mode::Hunger)
        .into_iter()
        .next()?;
    let host = host_of(&url)?;
    let identity = state.catalog().transfer_identity(post, &url)?;
    let context = StartupContext::new(
        state.catalog().estimated_bitrate(post, state.params()),
        inputs.observed_at_ms,
        state.params().head_seconds,
    );
    Some(SourceChoice {
        url,
        identity,
        head_seconds: startup_seconds(inputs.stats, &host, context),
    })
}

fn host_factor(
    urls: &HashMap<PostId, String>,
    post: &PostId,
    stats: &HostStats,
    mode: Mode,
) -> f64 {
    urls.get(post)
        .and_then(|url| host_of(url))
        .map(|host| stats.host_factor(&host, mode))
        .unwrap_or(1.0)
}
