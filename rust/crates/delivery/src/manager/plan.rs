//! One planning pass: on-disk byte ranges plus engine state in, the
//! ordered chunk-request list and per-post source URLs out.

use crate::manager::retry::RetryBook;
use crate::manager::state::DeliveryState;
use crate::playback_demand::DemandSignal;
use eviction::{protected_seed_eviction, EvictionInputs, ProtectedSeedEviction};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::inventory_controller::PresentRanges;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scoring::{next_work, ChunkRequest, NextWorkContext};
use ghostr_engine::tiers::DemandSignals;
use ghostr_engine::{ByteRange, PostId};
use playback::{playback_plan, PlaybackPlanInputs};
use sources::{host_factor, source_choices, SourceChoices};
pub(crate) use startup::{startup_seconds, StartupContext};
use std::collections::{HashMap, HashSet};

pub(crate) mod eviction;
pub(crate) mod playback;
mod sources;
mod startup;

/// Everything a planning pass reads besides the engine state.
pub(crate) struct PlanInputs<'a> {
    pub stats: &'a HostStats,
    pub retry: &'a RetryBook,
    pub present: &'a HashMap<PostId, Vec<ByteRange>>,
    pub demand: DemandSignals,
    pub observed_at_ms: u64,
    pub demanded: Option<DemandSignal>,
}

pub(crate) struct PlannedWork {
    pub transfers: Vec<PlannedTransfer>,
    pub protected_identities: HashSet<TransferIdentity>,
    pub emergency: bool,
    pub eviction: ProtectedSeedEviction,
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
        PlaybackPlanInputs {
            stats: inputs.stats,
            urls: &choices.urls,
            observed_at_ms: inputs.observed_at_ms,
            demanded_end: inputs.demanded.as_ref().map(|signal| signal.range.end),
        },
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
    let current = state.focus().current();
    let demanded = inputs
        .demanded
        .as_ref()
        .filter(|signal| current == Some(&signal.post))
        .map(|signal| signal.range);
    let present = current
        .and_then(|post| inputs.present.get(post))
        .map(Vec::as_slice)
        .unwrap_or_default();
    let eviction = protected_seed_eviction(EvictionInputs {
        gateway_demand: inputs.demand.gateway_demand,
        current_startable: inventory.current_startable(),
        demanded,
        present,
        phase: state.playback().observation().map(|value| value.phase()),
        playback_emergency: playback.emergency(),
        buffer_below_emergency: inputs.demand.buffer_below_emergency,
    });
    let present_of = |post: &PostId| inputs.present.get(post).cloned().unwrap_or_default();
    let factor_of = |post: &PostId| host_factor(&choices.urls, post, inputs.stats, inventory.mode);
    let tail_end = |post: &PostId| playback.tail_end(post);
    let media_window = |post: &PostId| playback.media_window(post);
    let direct_range = |post: &PostId| {
        inputs
            .demanded
            .as_ref()
            .filter(|signal| &signal.post == post)
            .map(|signal| signal.range)
    };
    let emergency = inputs.demand.gateway_demand
        || inputs.demand.buffer_below_emergency
        || playback.emergency();
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
        media_window: &media_window,
        direct_range: &direct_range,
    });
    let transfers = pair_requests(requests, &choices);
    let protected_identities = choices.identities.into_values().collect();
    PlannedWork {
        transfers,
        protected_identities,
        emergency,
        eviction,
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
