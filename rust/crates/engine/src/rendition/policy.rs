use super::risk::BufferRisk;
use super::{Rendition, RenditionId, RenditionSet};
use crate::playback::{BufferTarget, EstimateConfidence, NetworkConditions, PlaybackObservation};

// Separate margins are the hysteresis band: upgrades need durable surplus,
// while an active rendition survives small estimate movement.
const DEFAULT_DOWNGRADE_HEADROOM_MILLI: u16 = 1_100;
const DEFAULT_UPGRADE_HEADROOM_MILLI: u16 = 1_400;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityDecision {
    selected: Rendition,
}

impl QualityDecision {
    pub fn selected(&self) -> &Rendition {
        &self.selected
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualitySelectionInput {
    network: NetworkConditions,
    observation: PlaybackObservation,
    target: BufferTarget,
    current: Option<RenditionId>,
}

impl QualitySelectionInput {
    pub const fn new(
        network: NetworkConditions,
        observation: PlaybackObservation,
        target: BufferTarget,
        current: Option<RenditionId>,
    ) -> Self {
        Self {
            network,
            observation,
            target,
            current,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QualitySelectionPolicy {
    downgrade_headroom_milli: u16,
    upgrade_headroom_milli: u16,
}

impl Default for QualitySelectionPolicy {
    fn default() -> Self {
        Self {
            downgrade_headroom_milli: DEFAULT_DOWNGRADE_HEADROOM_MILLI,
            upgrade_headroom_milli: DEFAULT_UPGRADE_HEADROOM_MILLI,
        }
    }
}

impl QualitySelectionPolicy {
    pub fn select(
        self,
        renditions: &RenditionSet,
        input: QualitySelectionInput,
    ) -> QualityDecision {
        let risk = BufferRisk::from(input.observation, input.target);
        match renditions.find(input.current.as_ref()) {
            Some(current) => self.select_from_current(renditions, current, &input, risk),
            None => self.select_initial(renditions, &input, risk),
        }
    }

    fn select_initial(
        self,
        renditions: &RenditionSet,
        input: &QualitySelectionInput,
        risk: BufferRisk,
    ) -> QualityDecision {
        let limit = selection_limit(input, risk, self.upgrade_headroom_milli);
        decision(renditions.highest_at_or_below(limit))
    }

    fn select_from_current(
        self,
        renditions: &RenditionSet,
        current: &Rendition,
        input: &QualitySelectionInput,
        risk: BufferRisk,
    ) -> QualityDecision {
        if risk == BufferRisk::Frozen {
            return decision(current);
        }
        let safe = renditions.highest_at_or_below(selection_limit(
            input,
            risk,
            self.downgrade_headroom_milli,
        ));
        if safe.bitrate_bits_per_second() < current.bitrate_bits_per_second() {
            return decision(safe);
        }
        self.upgrade_or_hold(renditions, current, input, risk)
    }

    fn upgrade_or_hold(
        self,
        renditions: &RenditionSet,
        current: &Rendition,
        input: &QualitySelectionInput,
        risk: BufferRisk,
    ) -> QualityDecision {
        if risk != BufferRisk::Comfortable || input.network.confidence() != EstimateConfidence::High
        {
            return decision(current);
        }
        let candidate = renditions.highest_at_or_below(selection_limit(
            input,
            risk,
            self.upgrade_headroom_milli,
        ));
        if candidate.bitrate_bits_per_second() > current.bitrate_bits_per_second() {
            decision(candidate)
        } else {
            decision(current)
        }
    }
}

fn selection_limit(input: &QualitySelectionInput, risk: BufferRisk, headroom_milli: u16) -> u64 {
    let rate = u128::from(input.observation.playback_rate_milli());
    let sustainable = u128::from(input.network.sustainable_bits_per_second());
    let media_capacity = sustainable.saturating_mul(1_000) / rate;
    let limited = media_capacity.saturating_mul(u128::from(risk.capacity_milli()))
        / u128::from(headroom_milli);
    limited.min(u128::from(u64::MAX)) as u64
}

fn decision(selected: &Rendition) -> QualityDecision {
    QualityDecision {
        selected: selected.clone(),
    }
}
