use super::super::privacy::DecisionPrivacy;
use super::RecordedWarpAction;
use super::{command, kind, RecordedResourceCost, RecordedResourcePrices, RecordedTwinEvaluation};
use super::{RecordedWarpDecision, RecordedWarpReserve};
use crate::adaptive::{DecisionAction, WarpPlanningDecision};

const RECORD_LIMIT: usize = 64;

pub(in crate::adaptive::decision) struct WarpCapture {
    pub decision: RecordedWarpDecision,
    pub admissible_candidates: Vec<String>,
    pub chosen_action: Option<DecisionAction>,
    pub random_seed: u64,
}

pub(in crate::adaptive::decision) fn capture(
    value: &WarpPlanningDecision,
    privacy: &DecisionPrivacy,
) -> WarpCapture {
    let selected = value
        .selected
        .as_ref()
        .map(|item| action(item, value.prices, privacy));
    let chosen_action = selected.as_ref().map(project);
    let evaluation = value.evaluation.map(RecordedTwinEvaluation::from);
    let random_seed = evaluation.map_or(0, |item| item.common_random_seed);
    WarpCapture {
        admissible_candidates: admissible(value, privacy),
        chosen_action,
        random_seed,
        decision: RecordedWarpDecision {
            selected,
            admissible_action_ids: value.admissible_action_ids.clone(),
            prices: prices(value.prices),
            evaluation,
            reserve: RecordedWarpReserve::from(value.reserve),
            additional_request_slot_demanded: value.additional_request_slot_demanded,
        },
    }
}

fn action(
    value: &crate::adaptive::GeneratedAction,
    prices: crate::adaptive::ResourcePrices,
    privacy: &DecisionPrivacy,
) -> RecordedWarpAction {
    let resources = value.node.resources;
    RecordedWarpAction {
        planner_action_id: value.node.id,
        post_id: privacy.post(value.node.post.as_str()),
        kind: kind::capture(&value.node.kind, privacy),
        command: command::capture(&value.command, privacy),
        resources: RecordedResourceCost::from(resources),
        dependencies: value.node.requires.clone(),
        ready_playback_ms: value.node.forecast.ready_playback_ms,
        static_score_micros: value.node.value.total(resources, prices),
    }
}

fn admissible(value: &WarpPlanningDecision, privacy: &DecisionPrivacy) -> Vec<String> {
    let mut posts = Vec::new();
    for action in &value.generated.actions {
        if value.admissible_action_ids.contains(&action.node.id) {
            let post = privacy.post(action.node.post.as_str());
            if !posts.contains(&post) && posts.len() < RECORD_LIMIT {
                posts.push(post);
            }
        }
    }
    posts
}

fn project(action: &RecordedWarpAction) -> DecisionAction {
    let (request, source, start, end) = action.command.projection();
    DecisionAction {
        post_id: action.post_id.clone(),
        source_id: source.into(),
        request: request.into(),
        bytes_start: start,
        bytes_end: end,
        expected_playable_gain_ms: action.ready_playback_ms,
        utility_micros: action.static_score_micros,
        reason: "WarpSelected".into(),
        retained: false,
    }
}

fn prices(value: crate::adaptive::ResourcePrices) -> RecordedResourcePrices {
    RecordedResourcePrices {
        network_micros: value.network_micros,
        storage_micros: value.storage_micros,
        cpu_micros: value.cpu_micros,
        request_micros: value.request_micros,
    }
}

impl From<crate::adaptive::ResourceCost> for RecordedResourceCost {
    fn from(value: crate::adaptive::ResourceCost) -> Self {
        Self {
            network_bytes: value.network_bytes,
            storage_bytes: value.storage_bytes,
            cpu_ms: value.cpu_ms,
            requests: value.requests,
        }
    }
}

impl From<crate::adaptive::TwinEvaluation> for RecordedTwinEvaluation {
    fn from(value: crate::adaptive::TwinEvaluation) -> Self {
        Self {
            expected_score_micros: value.expected_score_micros,
            expected_visible_delay_ms: value.expected_visible_delay_ms,
            p95_visible_delay_ms: value.p95_visible_delay_ms,
            p99_visible_delay_ms: value.p99_visible_delay_ms,
            cvar_visible_delay_ms: value.cvar_visible_delay_ms,
            on_time_probability_bps: value.on_time_probability_bps,
            expected_ready_coverage_ms: value.expected_ready_coverage_ms,
            expected_cache_bytes: value.expected_cache_bytes,
            common_random_seed: value.common_random_seed,
        }
    }
}

impl From<crate::adaptive::ReserveConstraint> for RecordedWarpReserve {
    fn from(value: crate::adaptive::ReserveConstraint) -> Self {
        Self {
            reserved_request_slots: value.reserved_request_slots,
            reserved_network_bytes: value.reserved_network_bytes,
            degraded: value.degraded,
        }
    }
}
