use super::super::{ActionForecast, ActionKind};
use super::builder::Builder;
use super::prediction::{
    basis_points, cvar, estimate_open_body, ready_gain, transfer_ms, Prediction, PredictionInput,
};
use crate::adaptive::{CandidateSnapshot, CompletionTimes};

#[derive(Clone, Copy)]
pub(super) struct Target<'a> {
    action: &'a ActionKind,
    source: &'a str,
    unread_body_bytes: u64,
    request_profile: crate::origin_model::OriginRequestProfile,
}

impl<'a> Target<'a> {
    pub(super) const fn new(
        action: &'a ActionKind,
        source: &'a str,
        unread_body_bytes: u64,
        request_profile: crate::origin_model::OriginRequestProfile,
    ) -> Self {
        Self {
            action,
            source,
            unread_body_bytes,
            request_profile,
        }
    }
}

pub(super) fn predict(
    builder: &Builder<'_>,
    candidate: &CandidateSnapshot,
    target: Target<'_>,
) -> Prediction {
    let input = prediction_input(builder, candidate, &target);
    let estimate = estimate_open_body(input, request_profile(candidate, &target));
    Prediction {
        forecast: ActionForecast::new(
            completion(target.unread_body_bytes, &estimate),
            basis_points(estimate.success.selected),
            ready_gain(
                candidate,
                target.action,
                builder.base,
                builder.direct_playback_blocked(candidate),
            ),
        ),
        uncertainty_bps: basis_points(estimate.uncertainty),
        request_profile: None,
    }
}

fn prediction_input<'a>(
    builder: &'a Builder<'_>,
    candidate: &'a CandidateSnapshot,
    target: &'a Target<'a>,
) -> PredictionInput<'a> {
    PredictionInput {
        model: builder.origins,
        snapshot: builder.snapshot,
        base: builder.base,
        candidate,
        action: target.action,
        source: target.source,
        concurrency: builder
            .context
            .request_occupancy()
            .authority_count(target.source),
        mode: builder.base.mode,
        direct_playback_blocked: builder.direct_playback_blocked(candidate),
        network_class: builder.context.network_class(),
    }
}

fn request_profile(
    _candidate: &CandidateSnapshot,
    target: &Target<'_>,
) -> crate::origin_model::OriginRequestProfile {
    target.request_profile
}

fn completion(bytes: u64, estimate: &crate::origin_model::OriginEstimate) -> CompletionTimes {
    let expected = transfer_ms(bytes, estimate.throughput_bps.p50);
    let p95 = transfer_ms(bytes, estimate.throughput_bps.p10).max(expected);
    let tail_rate = estimate
        .throughput_bps
        .p10
        .min(estimate.throughput_bps.selected)
        .max(1);
    let p99 = transfer_ms(bytes, tail_rate).max(p95);
    CompletionTimes::new(expected, p95, p99, cvar(p95, p99))
}
