use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CapabilityAttempt {
    pub(super) client_epoch: u64,
    pub(super) attempt_generation: u64,
}

impl CapabilityAttempt {
    pub(crate) const fn new(client_epoch: u64, attempt_generation: u64) -> Self {
        Self {
            client_epoch,
            attempt_generation,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ClientCapabilityProfile {
    representation: String,
    codec: Option<String>,
    dimensions: Option<(u32, u32)>,
}

impl ClientCapabilityProfile {
    pub(crate) fn try_new(
        representation: &str,
        codec: Option<&str>,
        dimensions: Option<(u32, u32)>,
    ) -> Result<Self, CapabilityProfileError> {
        let representation = required(representation)?;
        let codec = codec
            .map(required)
            .transpose()?
            .map(|value| value.to_lowercase());
        if dimensions.is_some_and(|(width, height)| width == 0 || height == 0) {
            return Err(CapabilityProfileError::ZeroDimension);
        }
        Ok(Self {
            representation,
            codec,
            dimensions,
        })
    }

    pub(crate) fn codec(&self) -> Option<&str> {
        self.codec.as_deref()
    }

    pub(crate) const fn dimensions(&self) -> Option<(u32, u32)> {
        self.dimensions
    }

    pub(super) fn is_valid(&self) -> bool {
        !self.representation.trim().is_empty()
            && self
                .codec
                .as_ref()
                .is_none_or(|value| !value.trim().is_empty())
            && self
                .dimensions
                .is_none_or(|(width, height)| width > 0 && height > 0)
    }
}

fn required(value: &str) -> Result<String, CapabilityProfileError> {
    let value = value.trim();
    if value.is_empty() {
        Err(CapabilityProfileError::EmptyValue)
    } else {
        Ok(value.to_owned())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapabilityProfileError {
    EmptyValue,
    ZeroDimension,
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

#[derive(Clone, Debug, Eq, PartialEq)]
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
