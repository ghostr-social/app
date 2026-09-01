mod codes;
mod origin;

use super::request::{RangeState, RequestState, StartupState};
use crate::adaptive::{CandidateSnapshot, InFlightAction, PlayableRange, PromotionOpportunity};
use crate::evidence::EvidenceAssessment;
use codes::{layout_code, preparation_code};
use origin::OriginState;
use serde::{Deserialize, Serialize};

use super::super::privacy::DecisionPrivacy;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct CandidateState {
    post: String,
    feed_offset: i32,
    view_probability: f64,
    retrieval_eligible: bool,
    total_bytes: Option<u64>,
    bitrate_bps: u64,
    duration_ms: u64,
    layout: u8,
    preferred_source: Option<String>,
    startup: Option<StartupState>,
    player_preparation: u8,
    #[serde(default, skip_serializing_if = "is_false")]
    direct_playback_blocked: bool,
    timeline_probe: Option<PlayableState>,
    playable_ranges: Vec<PlayableState>,
    demanded: Option<RangeState>,
    present: Vec<RangeState>,
    finalized: bool,
    recently_evicted: Vec<RangeState>,
    in_flight: Vec<InFlightState>,
    origins: Vec<OriginState>,
    evidence: EvidenceAssessment,
}

impl CandidateState {
    pub(super) const fn direct_playback_blocked(&self) -> bool {
        self.direct_playback_blocked
    }

    pub(super) fn capture(value: &CandidateSnapshot, privacy: &DecisionPrivacy) -> Self {
        Self {
            post: privacy.post(value.post.as_str()),
            feed_offset: value.feed_offset.value(),
            view_probability: value.view_probability.value(),
            retrieval_eligible: value.retrieval_eligible,
            total_bytes: value.total_bytes,
            bitrate_bps: value.bitrate_bps,
            duration_ms: value.duration_ms,
            layout: layout_code(value.layout),
            preferred_source: value
                .preferred_source
                .as_deref()
                .map(|item| privacy.source(item)),
            startup: value.startup.as_ref().map(StartupState::capture),
            player_preparation: preparation_code(value.player_preparation),
            direct_playback_blocked: value.direct_playback_blocked,
            timeline_probe: value.timeline_probe.map(PlayableState::capture),
            playable_ranges: map_playable(&value.playable_ranges),
            demanded: value.demanded.map(RangeState::capture),
            present: map_ranges(&value.present),
            finalized: value.finalized,
            recently_evicted: map_ranges(&value.recently_evicted),
            in_flight: value
                .in_flight
                .iter()
                .map(|item| InFlightState::capture(item, privacy))
                .collect(),
            origins: value
                .origins
                .iter()
                .map(|item| OriginState::capture(item, privacy))
                .collect(),
            evidence: value.evidence.clone(),
        }
    }
}

const fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct PlayableState {
    bytes: RangeState,
    playable_ms: u64,
}

impl PlayableState {
    fn capture(value: PlayableRange) -> Self {
        Self {
            bytes: RangeState::capture(value.bytes),
            playable_ms: value.playable_ms,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct InFlightState {
    action_id: u64,
    request: RequestState,
    effective_bytes: RangeState,
    reserved_storage_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    promotion_opportunity: Option<PromotionOpportunity>,
    source: String,
    committed_until_ms: u64,
    identity_current: bool,
    cancelling: bool,
}

impl InFlightState {
    fn capture(value: &InFlightAction, privacy: &DecisionPrivacy) -> Self {
        Self {
            action_id: value.action_id.value(),
            request: RequestState::capture(value.request),
            effective_bytes: RangeState::capture(value.effective_bytes),
            reserved_storage_bytes: value.reserved_storage_bytes,
            promotion_opportunity: value.promotion_opportunity,
            source: privacy.source(&value.source),
            committed_until_ms: value.committed_until_ms,
            identity_current: value.identity_current,
            cancelling: value.cancelling,
        }
    }
}

fn map_playable(values: &[PlayableRange]) -> Vec<PlayableState> {
    values.iter().copied().map(PlayableState::capture).collect()
}

fn map_ranges(values: &[crate::ByteRange]) -> Vec<RangeState> {
    values.iter().copied().map(RangeState::capture).collect()
}
