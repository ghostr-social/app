use serde::{Deserialize, Serialize};

mod profile;
pub(crate) use profile::ClientCapabilityProfile;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CapabilityAttempt {
    client_epoch: u64,
    attempt_generation: u64,
}

impl CapabilityAttempt {
    pub(crate) const fn new(client_epoch: u64, attempt_generation: u64) -> Self {
        Self {
            client_epoch,
            attempt_generation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapabilitySignal {
    Initializing,
    FirstFrameRendered,
    UnsupportedFailure,
    InconclusiveFailure,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CapabilityEvent {
    pub(super) observed_us: u64,
    pub(super) signal: CapabilitySignal,
}

impl CapabilityEvent {
    pub(crate) const fn new(observed_us: u64, signal: CapabilitySignal) -> Self {
        Self {
            observed_us,
            signal,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CapabilityObservation {
    pub(super) capability_generation: u64,
    pub(super) attempt: CapabilityAttempt,
    pub(super) profile: ClientCapabilityProfile,
    pub(super) event: CapabilityEvent,
}

impl CapabilityObservation {
    pub(crate) const fn new(
        capability_generation: u64,
        attempt: CapabilityAttempt,
        profile: ClientCapabilityProfile,
        event: CapabilityEvent,
    ) -> Self {
        Self {
            capability_generation,
            attempt,
            profile,
            event,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClientCapabilityStatus {
    Unknown,
    Testing,
    Supported { p95_first_frame_us: u64 },
    Unsupported,
    Inconclusive,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ClientCapabilityState {
    pub(super) generation: Option<u64>,
    pub(super) records: Vec<CapabilityRecord>,
    #[serde(default)]
    pub(super) revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct CapabilityRecord {
    pub(super) profile: ClientCapabilityProfile,
    pub(super) result: CapabilityResult,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) enum CapabilityResult {
    Supported { first_frame_us: Vec<u64> },
    Unsupported,
    Inconclusive,
}
