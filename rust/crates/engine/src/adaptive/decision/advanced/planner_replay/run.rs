use super::{
    RecordedNetworkState, RecordedPlannerConfig, RecordedPlannerReplayCapsule, MODEL_BYTE_LIMIT,
};
use crate::adaptive::decision::privacy::DecisionPrivacy;
use crate::adaptive::{
    DecisionReplayStatus, NetworkTokenBucket, PlannerReplayCapsule, PlannerReplayState,
    ResourcePrices, WarpPlannerConfig,
};

pub(in crate::adaptive::decision) fn verify(
    capsule: &RecordedPlannerReplayCapsule,
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    expected: &super::super::RecordedWarpDecision,
) -> Result<(), DecisionReplayStatus> {
    let actual = capsule.restore()?.run(snapshot);
    let mut recorded = super::super::capture(&actual, &DecisionPrivacy::passthrough()).decision;
    let mut expected = expected.clone();
    recorded.planner_replay_capsule = None;
    expected.planner_replay_capsule = None;
    require(recorded == expected)
}

impl RecordedPlannerReplayCapsule {
    fn restore(&self) -> Result<PlannerReplayCapsule, DecisionReplayStatus> {
        require_available(self.complete)?;
        let base = self.base.as_ref().ok_or(unavailable())?;
        let context = self.context.as_ref().ok_or(unavailable())?;
        let origins = self.origins.as_ref().ok_or(unavailable())?;
        require(
            base.replay_bounded()
                && context.replay_bounded()
                && origins.replay_bounded()
                && serde_json::to_vec(origins).is_ok_and(|value| value.len() <= MODEL_BYTE_LIMIT),
        )?;
        let config = self.config.ok_or(unavailable())?.restore()?;
        let controller_prices = self.controller_prices.ok_or(unavailable())?.into();
        let network = self.network.ok_or(unavailable())?.restore();
        let price_epoch = self.price_epoch.ok_or(unavailable())?;
        Ok(PlannerReplayCapsule::restored(
            base.clone(),
            origins.clone(),
            context.clone(),
            PlannerReplayState {
                config,
                controller_prices,
                network,
                price_epoch,
                last_feedback: self.last_feedback,
                generation_policies: self.generation_policies(),
            },
            Vec::new(),
        ))
    }
}

impl RecordedPlannerConfig {
    fn restore(self) -> Result<WarpPlannerConfig, DecisionReplayStatus> {
        Ok(WarpPlannerConfig {
            beam: crate::adaptive::BeamConfig::new(
                usize::try_from(self.beam_depth).map_err(|_conversion_error| mismatch())?,
                usize::try_from(self.beam_width).map_err(|_conversion_error| mismatch())?,
                usize::try_from(self.beam_expansions).map_err(|_conversion_error| mismatch())?,
                self.beam_latency_us,
            ),
            twin: crate::adaptive::TwinConfig::new(self.twin_particles, self.twin_tail_bps),
            semantic_top_k: usize::try_from(self.semantic_top_k)
                .map_err(|_conversion_error| mismatch())?,
            semantic_epsilon_micros: self.semantic_epsilon_micros,
            safety_rescue_bps: self.safety_rescue_bps,
            emergency_rescue_bps: self.emergency_rescue_bps,
        })
    }
}

impl RecordedNetworkState {
    fn restore(self) -> Option<NetworkTokenBucket> {
        match self {
            Self::Uninitialized => None,
            Self::Initialized {
                capacity,
                refill_per_second,
                tokens,
                updated_at_ms,
                refill_milli_bytes,
                debt_bytes,
            } => Some(NetworkTokenBucket::from_replay((
                capacity,
                refill_per_second,
                tokens,
                updated_at_ms,
                refill_milli_bytes,
                debt_bytes,
            ))),
        }
    }
}

impl From<super::super::RecordedResourcePrices> for ResourcePrices {
    fn from(value: super::super::RecordedResourcePrices) -> Self {
        Self {
            network_micros: value.network_micros,
            storage_micros: value.storage_micros,
            cpu_micros: value.cpu_micros,
            request_micros: value.request_micros,
        }
    }
}

fn require(value: bool) -> Result<(), DecisionReplayStatus> {
    value.then_some(()).ok_or_else(mismatch)
}

fn require_available(value: bool) -> Result<(), DecisionReplayStatus> {
    value.then_some(()).ok_or_else(unavailable)
}

const fn unavailable() -> DecisionReplayStatus {
    DecisionReplayStatus::AdvancedReplayUnavailable
}

const fn mismatch() -> DecisionReplayStatus {
    DecisionReplayStatus::PlanMismatch
}
