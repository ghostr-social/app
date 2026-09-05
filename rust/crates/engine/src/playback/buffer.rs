use super::continuation::predicted_requirement;
use super::{MediaConsumption, NetworkConditions};
use core::time::Duration;

const MINIMUM: Duration = Duration::from_secs(4);
const MAXIMUM: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BufferTarget {
    steady: Duration,
    required: Duration,
    emergency_horizon: Duration,
}

impl BufferTarget {
    pub(crate) fn steady(self) -> Duration {
        self.steady
    }
    pub(crate) fn emergency_horizon(self) -> Duration {
        self.emergency_horizon
    }

    /// Conditional requirement under the conservative continuation scenario.
    /// It is never clipped to a retention cap or treated as a readiness proof.
    pub fn required(self) -> Duration {
        self.required
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdaptiveBufferPolicy {
    pub(super) minimum: Duration,
    maximum: Duration,
}

impl Default for AdaptiveBufferPolicy {
    fn default() -> Self {
        Self {
            minimum: MINIMUM,
            maximum: MAXIMUM,
        }
    }
}

impl AdaptiveBufferPolicy {
    pub fn target_for(
        self,
        network: NetworkConditions,
        media: MediaConsumption,
        remaining_media: Duration,
    ) -> BufferTarget {
        let required = predicted_requirement(network, media, remaining_media)
            .max(self.minimum.min(remaining_media));
        self.with_requirement(required, network)
    }

    pub(super) fn with_requirement(
        self,
        required: Duration,
        network: NetworkConditions,
    ) -> BufferTarget {
        BufferTarget {
            steady: required.min(self.maximum),
            required,
            emergency_horizon: (network.ttfb.saturating_mul(2) + Duration::from_secs(2))
                .min(required),
        }
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "buffer/test_support.rs"]
mod test_support;

impl BufferTarget {
    pub(crate) fn fits_retention_limit(self) -> bool {
        self.required() <= self.steady
    }
}
