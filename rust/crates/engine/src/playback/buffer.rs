use super::{EstimateConfidence, MediaConsumption, NetworkConditions};
use std::time::Duration;

const MINIMUM_SECONDS: u64 = 4;
const MAXIMUM_SECONDS: u64 = 30;
const STARTUP_MINIMUM_SECONDS: u64 = 2;
const STARTUP_MAXIMUM_SECONDS: u64 = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BufferTarget {
    startup: Duration,
    steady: Duration,
    emergency_horizon: Duration,
}

impl BufferTarget {
    pub fn startup(self) -> Duration {
        self.startup
    }

    pub fn steady(self) -> Duration {
        self.steady
    }

    pub fn emergency_horizon(self) -> Duration {
        self.emergency_horizon
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdaptiveBufferPolicy {
    minimum: Duration,
    maximum: Duration,
}

impl Default for AdaptiveBufferPolicy {
    fn default() -> Self {
        Self {
            minimum: Duration::from_secs(MINIMUM_SECONDS),
            maximum: Duration::from_secs(MAXIMUM_SECONDS),
        }
    }
}

impl AdaptiveBufferPolicy {
    pub fn target(self, network: NetworkConditions, media: MediaConsumption) -> BufferTarget {
        let risk = Risk::from(network, media);
        let steady = self.steady_target(risk);
        BufferTarget {
            startup: startup_target(risk, steady),
            steady,
            emergency_horizon: emergency_horizon(risk),
        }
    }

    pub fn maximum(self) -> Duration {
        self.maximum
    }

    fn steady_target(self, risk: Risk) -> Duration {
        if !risk.sustainable() {
            return self.maximum;
        }
        let seconds = MINIMUM_SECONDS as f64
            + risk.latency_s * 2.0
            + risk.refill_delay()
            + risk.variability_surcharge()
            + risk.confidence_surcharge();
        clamp_duration(seconds.ceil(), self.minimum, self.maximum)
    }
}

#[derive(Clone, Copy)]
struct Risk {
    safe_throughput: f64,
    consumption: f64,
    variability_ratio: f64,
    latency_s: f64,
    confidence: EstimateConfidence,
}

impl Risk {
    fn from(network: NetworkConditions, media: MediaConsumption) -> Self {
        let throughput = network.bytes_per_second.saturating_mul(8) as f64;
        let variability = network.variability_bytes_per_second.saturating_mul(8) as f64;
        let confidence_scale = confidence_scale(network.confidence);
        Self {
            safe_throughput: (throughput - variability * confidence_scale).max(0.0),
            consumption: media.bits_per_second() as f64,
            variability_ratio: variability / throughput.max(1.0),
            latency_s: network.ttfb.as_secs_f64(),
            confidence: network.confidence,
        }
    }

    fn sustainable(self) -> bool {
        self.safe_throughput > self.consumption
    }

    fn refill_delay(self) -> f64 {
        self.latency_s * self.consumption / (self.safe_throughput - self.consumption)
    }

    fn variability_surcharge(self) -> f64 {
        self.variability_ratio.min(1.0) * 8.0
    }

    fn confidence_surcharge(self) -> f64 {
        match self.confidence {
            EstimateConfidence::Low => 4.0,
            EstimateConfidence::Medium => 2.0,
            EstimateConfidence::High => 0.0,
        }
    }
}

fn startup_target(risk: Risk, steady: Duration) -> Duration {
    let sustainability = (!risk.sustainable()) as u8 as f64 * 2.0;
    let seconds = STARTUP_MINIMUM_SECONDS as f64
        + risk.latency_s
        + risk.variability_surcharge() / 4.0
        + risk.confidence_surcharge() / 2.0
        + sustainability;
    let maximum = Duration::from_secs(STARTUP_MAXIMUM_SECONDS)
        .min(steady.saturating_sub(Duration::from_secs(1)));
    Duration::from_secs(seconds.ceil() as u64).min(maximum)
}

fn emergency_horizon(risk: Risk) -> Duration {
    let seconds = 2.0
        + risk.latency_s * 2.0
        + risk.variability_surcharge()
        + risk.confidence_surcharge() / 2.0;
    Duration::from_secs_f64(seconds.min(MAXIMUM_SECONDS as f64))
}

fn confidence_scale(confidence: EstimateConfidence) -> f64 {
    match confidence {
        EstimateConfidence::Low => 2.0,
        EstimateConfidence::Medium => 1.5,
        EstimateConfidence::High => 1.0,
    }
}

fn clamp_duration(seconds: f64, minimum: Duration, maximum: Duration) -> Duration {
    Duration::from_secs(seconds as u64).clamp(minimum, maximum)
}
