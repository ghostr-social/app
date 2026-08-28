use super::super::super::privacy::DecisionPrivacy;
use super::{
    RecordedActionForecast, RecordedActionValue, RecordedAuthorityOccupancy, RecordedBeamConfig,
    RecordedCompletionTimes, RecordedSearchAction, RecordedSearchBudget, RecordedSearchReplayMode,
    RecordedSearchScore, RecordedWarpSearchInput,
};
use crate::adaptive::warp::{SearchReplayInput, SearchReplayMode};
use crate::adaptive::{ActionNode, WarpPlanningDecision};

const ACTION_LIMIT: usize = 64;
const SCORE_LIMIT: usize = 4_096;

pub(crate) fn capture(
    value: &WarpPlanningDecision,
    privacy: &DecisionPrivacy,
) -> Option<RecordedWarpSearchInput> {
    let input = value.search_replay.as_ref()?;
    if input.nodes().len() > ACTION_LIMIT || input.scores().len() > SCORE_LIMIT {
        return None;
    }
    Some(RecordedWarpSearchInput {
        mode: replay_mode(input.mode()),
        beam: beam(input)?,
        prices: recorded_prices(input.prices()),
        budget: budget(input, privacy)?,
        actions: input
            .nodes()
            .iter()
            .map(|node| action(node, privacy))
            .collect(),
        scores: scores(input),
        reserve: Some(super::super::reserve::capture(input.reserve(), privacy)),
        reserve_threshold_bps: input.reserve_threshold_bps(),
        reserve_degraded_reason: input.reserve_degraded_reason().map(Into::into),
    })
}

fn recorded_prices(value: crate::adaptive::ResourcePrices) -> super::RecordedResourcePrices {
    super::RecordedResourcePrices {
        network_micros: value.network_micros,
        storage_micros: value.storage_micros,
        cpu_micros: value.cpu_micros,
        request_micros: value.request_micros,
    }
}

fn replay_mode(value: SearchReplayMode) -> RecordedSearchReplayMode {
    match value {
        SearchReplayMode::Beam => RecordedSearchReplayMode::Beam,
        SearchReplayMode::GreedyExpansion => RecordedSearchReplayMode::GreedyExpansion,
        SearchReplayMode::GreedyLatency => RecordedSearchReplayMode::GreedyLatency,
        SearchReplayMode::LeastRisk => RecordedSearchReplayMode::LeastRisk,
    }
}

fn beam(input: &SearchReplayInput) -> Option<RecordedBeamConfig> {
    let (depth, width, expansions, latency) = input.beam().replay_parts();
    Some(RecordedBeamConfig {
        depth: u64::try_from(depth).ok()?,
        width: u64::try_from(width).ok()?,
        max_expansions: u64::try_from(expansions).ok()?,
        max_latency_us: latency,
    })
}

fn budget(input: &SearchReplayInput, privacy: &DecisionPrivacy) -> Option<RecordedSearchBudget> {
    let value = input.budget();
    Some(RecordedSearchBudget {
        remaining: value.replay_remaining().into(),
        segmented_storage_bytes: value
            .replay_segmented_storage()
            .map(|storage| storage.available_bytes()),
        global_request_width: Some(value.replay_request_width()),
        per_origin_requests: u64::try_from(value.replay_per_origin_requests()).ok()?,
        origins: sorted_occupancy(
            value
                .replay_origins()
                .iter()
                .map(|(origin, count)| occupancy(origin.as_str(), *count, privacy))
                .collect::<Option<Vec<_>>>()?,
        ),
        pending_rescue_action_ids: value.replay_pending().iter().map(|node| node.id).collect(),
    })
}

fn occupancy(
    source: &str,
    requests: usize,
    privacy: &DecisionPrivacy,
) -> Option<RecordedAuthorityOccupancy> {
    Some(RecordedAuthorityOccupancy {
        source_id: privacy.authority(source),
        requests: u64::try_from(requests).ok()?,
    })
}

fn sorted_occupancy(
    mut values: Vec<RecordedAuthorityOccupancy>,
) -> Vec<RecordedAuthorityOccupancy> {
    values.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    values
}

fn action(node: &ActionNode, privacy: &DecisionPrivacy) -> RecordedSearchAction {
    RecordedSearchAction {
        planner_action_id: node.id,
        post_id: privacy.post(node.post.as_str()),
        kind: super::super::kind::capture(&node.kind, privacy),
        value: node.value.into(),
        resources: node.resources.into(),
        authorized_resources: node.resource_authority().map(Into::into),
        origin_admission_intent: node.origin_admission_intent().into(),
        forecast: node.forecast.into(),
        request_source_id: node
            .request_authority()
            .map(|_| privacy.source(node.replay_origin())),
        dependencies: node.requires.clone(),
    }
}

fn scores(input: &SearchReplayInput) -> Vec<RecordedSearchScore> {
    input
        .scores()
        .iter()
        .map(|score| RecordedSearchScore {
            action_ids: score.action_ids.clone(),
            score_micros: score.score_micros,
        })
        .collect()
}

impl From<crate::adaptive::ActionValue> for RecordedActionValue {
    fn from(value: crate::adaptive::ActionValue) -> Self {
        Self {
            delay_loss_micros: value.delay_loss_micros,
            reserve_gain_micros: value.reserve_gain_micros,
            information_value_micros: value.information_value_micros,
            exploration_micros: value.exploration_micros,
            cache_gain_micros: value.cache_gain_micros,
            tail_risk_micros: value.tail_risk_micros,
            cvar_micros: value.cvar_micros,
            rank_cost_micros: value.rank_cost_micros,
        }
    }
}

impl From<crate::adaptive::ActionForecast> for RecordedActionForecast {
    fn from(value: crate::adaptive::ActionForecast) -> Self {
        Self {
            completion: RecordedCompletionTimes {
                expected_ms: value.completion.expected_ms,
                p95_ms: value.completion.p95_ms,
                p99_ms: value.completion.p99_ms,
                cvar_ms: value.completion.cvar_ms,
            },
            success_bps: value.success_bps,
            ready_playback_ms: value.ready_playback_ms,
            quality_gain_micros: value.quality_gain_micros,
            cache_reuse_bps: value.cache_reuse_bps,
        }
    }
}
