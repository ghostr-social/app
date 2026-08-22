mod codes;
mod origin;

use super::request::{RangeState, RequestState, StartupState};
use crate::adaptive::{
    CandidateSnapshot, FeedOffset, InFlightAction, PlayableRange, ViewProbability,
};
use crate::evidence::EvidenceAssessment;
use crate::{ActionId, PostId};
use codes::{layout, layout_code, preparation, preparation_code};
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

    pub(super) fn snapshot(&self) -> CandidateSnapshot {
        CandidateSnapshot {
            post: PostId::new(&self.post),
            feed_offset: FeedOffset::new(self.feed_offset),
            view_probability: ViewProbability::new(self.view_probability)
                .expect("captured probability remains valid"),
            retrieval_eligible: self.retrieval_eligible,
            total_bytes: self.total_bytes,
            bitrate_bps: self.bitrate_bps,
            duration_ms: self.duration_ms,
            layout: layout(self.layout),
            preferred_source: self.preferred_source.clone(),
            startup: self.startup.as_ref().map(StartupState::startup),
            player_preparation: preparation(self.player_preparation),
            timeline_probe: self.timeline_probe.map(PlayableState::playable),
            playable_ranges: self
                .playable_ranges
                .iter()
                .copied()
                .map(PlayableState::playable)
                .collect(),
            demanded: self.demanded.map(RangeState::range),
            present: restore_ranges(&self.present),
            finalized: self.finalized,
            recently_evicted: restore_ranges(&self.recently_evicted),
            in_flight: self.in_flight.iter().map(InFlightState::action).collect(),
            origins: self.origins.iter().map(OriginState::origin).collect(),
            evidence: self.evidence.clone(),
        }
    }
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

    fn playable(self) -> PlayableRange {
        PlayableRange {
            bytes: self.bytes.range(),
            playable_ms: self.playable_ms,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct InFlightState {
    action_id: u64,
    request: RequestState,
    effective_bytes: RangeState,
    reserved_storage_bytes: u64,
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
            source: privacy.source(&value.source),
            committed_until_ms: value.committed_until_ms,
            identity_current: value.identity_current,
            cancelling: value.cancelling,
        }
    }

    fn action(&self) -> InFlightAction {
        InFlightAction {
            action_id: ActionId::new(self.action_id),
            request: self.request.request(),
            effective_bytes: self.effective_bytes.range(),
            reserved_storage_bytes: self.reserved_storage_bytes,
            source: self.source.clone(),
            committed_until_ms: self.committed_until_ms,
            identity_current: self.identity_current,
            cancelling: self.cancelling,
        }
    }
}

fn map_playable(values: &[PlayableRange]) -> Vec<PlayableState> {
    values.iter().copied().map(PlayableState::capture).collect()
}

fn map_ranges(values: &[crate::ByteRange]) -> Vec<RangeState> {
    values.iter().copied().map(RangeState::capture).collect()
}

fn restore_ranges(values: &[RangeState]) -> Vec<crate::ByteRange> {
    values.iter().copied().map(RangeState::range).collect()
}
