mod compare;

use super::super::RecordedWarpSearch;
use super::{
    RecordedBeamConfig, RecordedSearchBudget, RecordedSearchReplayMode, RecordedSearchScore,
    RecordedWarpReserve, RecordedWarpSearchInput,
};
use crate::adaptive::warp::{ScoredSearchPlan, SearchReplayInput, SearchReplayMode};
use crate::adaptive::{
    ActionNode, BeamConfig, DecisionReplayStatus, HardBudget, ResourcePrices,
    SegmentedStorageBudget,
};
use crate::RequestAuthority;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
#[path = "run/progress_validation_test.rs"]
mod progress_validation_test;

pub(in crate::adaptive::decision) fn verify(
    input: &RecordedWarpSearchInput,
    expected: &RecordedWarpSearch,
    selected: Option<u16>,
    reserve: &RecordedWarpReserve,
) -> Result<(), DecisionReplayStatus> {
    super::reserve::verify(input, reserve)?;
    let replay = input.restore().ok_or(DecisionReplayStatus::PlanMismatch)?;
    let actual = replay.run().ok_or(DecisionReplayStatus::PlanMismatch)?;
    require(actual.action.as_ref().map(|action| action.id) == selected)?;
    require(compare::matches(&actual, expected))
}

impl RecordedWarpSearchInput {
    fn restore(&self) -> Option<SearchReplayInput> {
        let nodes = restore_actions(&self.actions)?;
        let mode = self.mode.restore();
        if mode != SearchReplayMode::Beam && !self.scores.is_empty() {
            return None;
        }
        if !valid_progress(mode, &self.reserve_progress_action_ids, &nodes) {
            return None;
        }
        Some(SearchReplayInput {
            mode,
            reserve: restore_reserve(self.reserve.as_ref())?,
            reserve_threshold_bps: self.reserve_threshold_bps,
            reserve_degraded_reason: self.reserve_degraded_reason.map(Into::into),
            budget: self.budget.restore(&nodes)?,
            beam: self.beam.restore()?,
            prices: restore_prices(self.prices),
            scores: self
                .scores
                .iter()
                .map(RecordedSearchScore::restore)
                .collect(),
            reserve_progress_action_ids: self.reserve_progress_action_ids.clone(),
            nodes,
        })
    }
}

fn valid_progress(mode: SearchReplayMode, ids: &[u16], nodes: &[ActionNode]) -> bool {
    if ids.is_empty() {
        return true;
    }
    if mode != SearchReplayMode::LeastRisk {
        return false;
    }
    let unique: BTreeSet<_> = ids.iter().copied().collect();
    unique.len() == ids.len() && ids.iter().all(|id| nodes.iter().any(|node| node.id == *id))
}

impl RecordedSearchReplayMode {
    const fn restore(self) -> SearchReplayMode {
        match self {
            Self::Beam => SearchReplayMode::Beam,
            Self::GreedyExpansion => SearchReplayMode::GreedyExpansion,
            Self::GreedyLatency => SearchReplayMode::GreedyLatency,
            Self::LeastRisk => SearchReplayMode::LeastRisk,
        }
    }
}

impl RecordedBeamConfig {
    fn restore(self) -> Option<BeamConfig> {
        Some(BeamConfig::new(
            usize::try_from(self.depth).ok()?,
            usize::try_from(self.width).ok()?,
            usize::try_from(self.max_expansions).ok()?,
            self.max_latency_us,
        ))
    }
}

impl RecordedSearchBudget {
    fn restore(&self, nodes: &[ActionNode]) -> Option<HardBudget> {
        let origins = restore_origins(&self.origins)?;
        let pending = self
            .pending_rescue_action_ids
            .iter()
            .map(|id| nodes.iter().find(|node| node.id == *id).cloned())
            .collect::<Option<Vec<_>>>()?;
        HardBudget::from_replay((
            self.remaining.restore(),
            self.global_request_width
                .unwrap_or_else(|| legacy_request_width(self)),
            usize::try_from(self.per_origin_requests).ok()?,
            origins,
            pending,
            self.segmented_storage_bytes
                .map(SegmentedStorageBudget::new),
        ))
    }
}

fn restore_reserve(
    value: Option<&RecordedWarpReserve>,
) -> Option<crate::adaptive::ReserveConstraint> {
    match value {
        Some(value) => super::super::reserve::restore(value),
        None => Some(Default::default()),
    }
}

fn legacy_request_width(value: &RecordedSearchBudget) -> u16 {
    value
        .origins
        .iter()
        .fold(value.remaining.requests, |total, item| {
            total.saturating_add(u16::try_from(item.requests).unwrap_or(u16::MAX))
        })
}

impl RecordedSearchScore {
    fn restore(&self) -> ScoredSearchPlan {
        ScoredSearchPlan {
            action_ids: self.action_ids.clone(),
            score_micros: self.score_micros,
        }
    }
}

fn restore_actions(values: &[super::RecordedSearchAction]) -> Option<Vec<ActionNode>> {
    let mut actions = Vec::with_capacity(values.len());
    for value in values {
        let action = value.restore()?;
        if actions
            .iter()
            .any(|prior: &ActionNode| prior.id == action.id)
        {
            return None;
        }
        actions.push(action);
    }
    Some(actions)
}

fn restore_origins(
    values: &[super::RecordedAuthorityOccupancy],
) -> Option<BTreeMap<RequestAuthority, usize>> {
    let mut origins = BTreeMap::new();
    for value in values {
        let authority = RequestAuthority::from_url(&value.source_id)?;
        let requests = usize::try_from(value.requests).ok()?;
        if origins.insert(authority, requests).is_some() {
            return None;
        }
    }
    Some(origins)
}

fn restore_prices(value: super::RecordedResourcePrices) -> ResourcePrices {
    ResourcePrices {
        network_micros: value.network_micros,
        storage_micros: value.storage_micros,
        cpu_micros: value.cpu_micros,
        request_micros: value.request_micros,
    }
}

fn require(value: bool) -> Result<(), DecisionReplayStatus> {
    value
        .then_some(())
        .ok_or(DecisionReplayStatus::PlanMismatch)
}
