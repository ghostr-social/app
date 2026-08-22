use super::super::{RecordedSearchAction, RecordedWarpSearchInput};
use crate::adaptive::{
    DecisionReplayStatus, RecordedRescueChanceEvidence, RecordedRescueTimingQuantile,
    RecordedWarpReserve,
};

const P95_FAILURE_BPS: u32 = 500;
const P99_FAILURE_BPS: u32 = 100;

pub(super) fn verify(
    input: &RecordedWarpSearchInput,
    reserve: &RecordedWarpReserve,
) -> Result<(), DecisionReplayStatus> {
    let Some(recorded) = reserve.chance else {
        return Ok(());
    };
    let actions = protected_actions(input, &reserve.protected_action_ids)?;
    let expected = chance(&actions, recorded).ok_or(DecisionReplayStatus::PlanMismatch)?;
    require(recorded == expected)
}

fn protected_actions<'a>(
    input: &'a RecordedWarpSearchInput,
    ids: &[u16],
) -> Result<Vec<&'a RecordedSearchAction>, DecisionReplayStatus> {
    ids.iter()
        .map(|id| {
            input
                .actions
                .iter()
                .find(|action| action.planner_action_id == *id)
                .ok_or(DecisionReplayStatus::PlanMismatch)
        })
        .collect()
}

fn chance(
    actions: &[&RecordedSearchAction],
    recorded: RecordedRescueChanceEvidence,
) -> Option<RecordedRescueChanceEvidence> {
    let transport_failure = transport_failure(actions);
    let allowed = 10_000_u32.saturating_sub(u32::from(recorded.threshold_bps));
    let timing = timing(actions, allowed.checked_sub(transport_failure)?)?;
    (timing.completion_ms <= recorded.deadline_ms).then_some(RecordedRescueChanceEvidence {
        achieved_success_bps: success_bps(transport_failure + timing.failure_bps),
        transport_success_bps: success_bps(transport_failure),
        timing_quantile: timing.quantile,
        timing_completion_ms: timing.completion_ms,
        ..recorded
    })
}

fn transport_failure(actions: &[&RecordedSearchAction]) -> u32 {
    actions.iter().fold(0_u32, |total, action| {
        total.saturating_add(u32::from(
            10_000_u16.saturating_sub(action.forecast.success_bps),
        ))
    })
}

fn timing(actions: &[&RecordedSearchAction], allowed: u32) -> Option<TimingEvidence> {
    let count = u32::try_from(actions.len()).ok()?;
    let (quantile, per_step) = quantile(count, allowed)?;
    Some(TimingEvidence {
        quantile,
        failure_bps: count.saturating_mul(per_step),
        completion_ms: completion(actions, quantile),
    })
}

fn quantile(count: u32, allowed: u32) -> Option<(RecordedRescueTimingQuantile, u32)> {
    if count.saturating_mul(P95_FAILURE_BPS) <= allowed {
        Some((RecordedRescueTimingQuantile::P95, P95_FAILURE_BPS))
    } else if count.saturating_mul(P99_FAILURE_BPS) <= allowed {
        Some((RecordedRescueTimingQuantile::P99, P99_FAILURE_BPS))
    } else {
        None
    }
}

fn completion(actions: &[&RecordedSearchAction], quantile: RecordedRescueTimingQuantile) -> u64 {
    actions.iter().fold(0_u64, |total, action| {
        let duration = match quantile {
            RecordedRescueTimingQuantile::P95 => action.forecast.completion.p95_ms,
            RecordedRescueTimingQuantile::P99 => action.forecast.completion.p99_ms,
        };
        total.saturating_add(duration)
    })
}

fn success_bps(failure_bps: u32) -> u16 {
    10_000_u32.saturating_sub(failure_bps).min(10_000) as u16
}

#[derive(Clone, Copy)]
struct TimingEvidence {
    quantile: RecordedRescueTimingQuantile,
    failure_bps: u32,
    completion_ms: u64,
}

fn require(value: bool) -> Result<(), DecisionReplayStatus> {
    value
        .then_some(())
        .ok_or(DecisionReplayStatus::PlanMismatch)
}
