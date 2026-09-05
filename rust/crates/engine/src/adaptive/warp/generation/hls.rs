use super::builder::Builder;
use super::HlsGenerationPolicy;
use crate::adaptive::{ActionValue, HlsBootstrapStage, HlsCandidateSnapshot};
use crate::adaptive::{ControlMode, ResourceCost};

mod action;

#[derive(Clone, Copy)]
struct HlsValueInput<'a> {
    candidate: &'a HlsCandidateSnapshot,
    stage: HlsBootstrapStage,
    prediction: super::prediction::Prediction,
    mode: ControlMode,
    expected_bytes: u64,
}

#[derive(Clone, Copy)]
struct HlsCommitment {
    maximum: u64,
    expected: u64,
    storage: u64,
    requests: u16,
    completes_object: bool,
}

impl HlsCommitment {
    const fn resources(self) -> ResourceCost {
        ResourceCost::new(self.expected, self.storage, 0, self.requests)
    }
}

pub(super) fn add(
    builder: &mut Builder<'_>,
    candidate: &HlsCandidateSnapshot,
    policy: HlsGenerationPolicy,
) {
    let Some((stage, source)) = candidate.pending() else {
        return;
    };
    let depth = if stage.is_manifest() { 8 } else { 2 };
    if !(0..=depth).contains(&candidate.feed_offset.value()) {
        return;
    }
    let Some(commitment) = commitment(builder, candidate, stage, policy) else {
        return;
    };
    if commitment.requests > 0 && !builder.context.permits_request(&candidate.post) {
        return;
    }
    action::push(
        builder,
        action::Input {
            candidate,
            stage,
            source,
            commitment,
        },
    );
}

fn commitment(
    builder: &Builder<'_>,
    candidate: &HlsCandidateSnapshot,
    stage: HlsBootstrapStage,
    policy: HlsGenerationPolicy,
) -> Option<HlsCommitment> {
    if policy == HlsGenerationPolicy::LegacyWholeStage {
        let maximum = stage.maximum_bytes();
        return Some(HlsCommitment {
            maximum,
            expected: maximum.min(builder.snapshot.request_slice_bytes),
            storage: maximum,
            requests: 1,
            completes_object: true,
        });
    }
    let maximum = candidate
        .cursor
        .block_bytes(stage, builder.snapshot.request_slice_bytes)?;
    Some(HlsCommitment {
        maximum,
        expected: maximum,
        storage: maximum,
        requests: u16::from(candidate.cursor.transport.opens_request()),
        completes_object: candidate.cursor.completes(maximum),
    })
}

fn value(input: HlsValueInput<'_>) -> ActionValue {
    let reach = (input.candidate.view_probability.value() * 10_000.0).round() as u64;
    let urgency = match input.mode {
        ControlMode::Emergency => 128,
        ControlMode::Safety => 96,
        ControlMode::Normal => 64,
    };
    let delay = input
        .candidate
        .startup_value_ms
        .saturating_mul(1_000)
        .saturating_mul(urgency)
        .saturating_mul(reach)
        / 10_000;
    ActionValue {
        delay_loss_micros: as_i64(delay),
        reserve_gain_micros: as_i64(stage_progress_gain(input.candidate, reach, urgency)),
        information_value_micros: as_i64(information(input.stage, reach, input.expected_bytes)),
        tail_risk_micros: as_i64(
            input
                .prediction
                .forecast
                .completion
                .p99_ms
                .saturating_mul(250),
        ),
        cvar_micros: as_i64(
            input
                .prediction
                .forecast
                .completion
                .cvar_ms
                .saturating_mul(100),
        ),
        rank_cost_micros: as_i64(input.candidate.feed_offset.magnitude() as u64 * 25_000),
        ..ActionValue::default()
    }
}

fn stage_progress_gain(candidate: &HlsCandidateSnapshot, reach: u64, urgency: u64) -> u64 {
    candidate
        .startup_value_ms
        .saturating_mul(1_000)
        .saturating_mul(urgency.saturating_add(32))
        .saturating_mul(reach)
        / 10_000
}

fn information(stage: HlsBootstrapStage, reach: u64, bytes: u64) -> u64 {
    if stage.is_manifest() {
        return bytes.saturating_mul(reach) / 10_000;
    }
    0
}

fn as_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}
