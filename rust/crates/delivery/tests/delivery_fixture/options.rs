//! Delivery-manager settings scaled for deterministic fixture servers.

use ghostr_delivery::manager::DeliveryTuning;
use ghostr_delivery::manager::retry::RetryPolicy;
use ghostr_engine::{DataUsageLevel, EngineParams};
use std::time::Duration;

pub struct DeliveryOptions {
    pub params: EngineParams,
    pub level: DataUsageLevel,
    pub tuning: DeliveryTuning,
}

impl Default for DeliveryOptions {
    fn default() -> Self {
        Self {
            params: base_params(),
            level: DataUsageLevel::Balanced,
            tuning: test_tuning(),
        }
    }
}

/// Small chunks and a tiny assumed bitrate so 16-byte fixture files
/// exercise real head/tail splits.
pub fn base_params() -> EngineParams {
    EngineParams {
        chunk_bytes: 8,
        assumed_bitrate_bps: 64,
        ..EngineParams::default()
    }
}

/// The production ladder, scaled down: pauses in milliseconds and a
/// revival window far longer than any test run.
fn test_tuning() -> DeliveryTuning {
    DeliveryTuning {
        probe_concurrency: 2,
        retry: RetryPolicy {
            base: Duration::from_millis(50),
            max: Duration::from_millis(400),
            jitter: 0.0,
            ..RetryPolicy::default()
        },
        stats_debounce: Duration::ZERO,
        store_pressure_pause: Duration::from_millis(10),
    }
}
