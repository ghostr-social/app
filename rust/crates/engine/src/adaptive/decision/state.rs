mod candidate;
mod codes;
mod request;

use super::privacy::DecisionPrivacy;
use candidate::CandidateState;
use codes::{authority, authority_code, confidence, confidence_code, phase, phase_code};
use serde::{Deserialize, Serialize};

use crate::adaptive::{
    NavigationSnapshot, NetworkSnapshot, PlayabilitySnapshot, PlaybackSnapshot, StorageSnapshot,
};
use crate::PostId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ReplayState {
    observed_at_ms: u64,
    commitment_ms: u64,
    request_slice_bytes: u64,
    playback: PlaybackState,
    network: NetworkState,
    storage: StorageState,
    navigation: NavigationState,
    candidates: Vec<CandidateState>,
}

impl ReplayState {
    pub(super) fn capture(value: &PlayabilitySnapshot, privacy: &DecisionPrivacy) -> Self {
        Self {
            observed_at_ms: value.observed_at_ms,
            commitment_ms: value.commitment_ms,
            request_slice_bytes: value.request_slice_bytes,
            playback: PlaybackState::capture(&value.playback, privacy),
            network: NetworkState::capture(value.network),
            storage: StorageState::capture(value.storage),
            navigation: NavigationState::capture(value.navigation),
            candidates: value
                .candidates
                .iter()
                .map(|item| CandidateState::capture(item, privacy))
                .collect(),
        }
    }

    pub(super) fn snapshot(&self) -> PlayabilitySnapshot {
        PlayabilitySnapshot {
            observed_at_ms: self.observed_at_ms,
            commitment_ms: self.commitment_ms,
            request_slice_bytes: self.request_slice_bytes,
            playback: self.playback.snapshot(),
            network: self.network.snapshot(),
            storage: StorageSnapshot::new(self.storage.budget_bytes, self.storage.used_bytes),
            navigation: self.navigation.snapshot(),
            candidates: self
                .candidates
                .iter()
                .map(CandidateState::snapshot)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct PlaybackState {
    current: String,
    authority: u8,
    phase: u8,
    buffer_ahead_ms: u64,
}

impl PlaybackState {
    fn capture(value: &PlaybackSnapshot, privacy: &DecisionPrivacy) -> Self {
        Self {
            current: privacy.post(value.current.as_str()),
            authority: authority_code(value.authority),
            phase: phase_code(value.phase),
            buffer_ahead_ms: value.buffer_ahead_ms,
        }
    }

    fn snapshot(&self) -> PlaybackSnapshot {
        PlaybackSnapshot {
            current: PostId::new(&self.current),
            authority: authority(self.authority),
            phase: phase(self.phase),
            buffer_ahead_ms: self.buffer_ahead_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct NetworkState {
    throughput_bps: u64,
    rtt_ms: u64,
    packet_loss_bps: u16,
    connection_capacity: usize,
    connection_ceiling: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    per_authority_request_limit: Option<usize>,
    confidence: u8,
}

impl NetworkState {
    fn capture(value: NetworkSnapshot) -> Self {
        Self {
            throughput_bps: value.throughput_bps,
            rtt_ms: value.rtt_ms,
            packet_loss_bps: value.packet_loss_bps,
            connection_capacity: value.connection_capacity,
            connection_ceiling: value.connection_ceiling,
            per_authority_request_limit: (value.per_authority_request_limit
                != value.connection_ceiling)
                .then_some(value.per_authority_request_limit),
            confidence: confidence_code(value.confidence),
        }
    }

    fn snapshot(self) -> NetworkSnapshot {
        NetworkSnapshot {
            throughput_bps: self.throughput_bps,
            rtt_ms: self.rtt_ms,
            packet_loss_bps: self.packet_loss_bps,
            connection_capacity: self.connection_capacity,
            connection_ceiling: self.connection_ceiling,
            per_authority_request_limit: self
                .per_authority_request_limit
                .unwrap_or(self.connection_ceiling),
            confidence: confidence(self.confidence),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct StorageState {
    budget_bytes: u64,
    used_bytes: u64,
}

impl StorageState {
    fn capture(value: StorageSnapshot) -> Self {
        Self {
            budget_bytes: value.budget_bytes,
            used_bytes: value.used_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct NavigationState {
    forward: u16,
    backward: u16,
}

impl NavigationState {
    fn capture(value: NavigationSnapshot) -> Self {
        Self {
            forward: value.forward_swipes_per_minute,
            backward: value.backward_swipes_per_minute,
        }
    }

    fn snapshot(self) -> NavigationSnapshot {
        NavigationSnapshot {
            forward_swipes_per_minute: self.forward,
            backward_swipes_per_minute: self.backward,
        }
    }
}
