//! Delivery-manager settings scaled for deterministic fixture servers.

use core::time::Duration;
use ghostr_delivery::manager::retry::RetryPolicy;
use ghostr_delivery::manager::DeliveryTuning;
use ghostr_engine::{DataUsageLevel, EngineParams};
use std::sync::Arc;

pub struct DeliveryOptions {
    pub params: EngineParams,
    pub level: DataUsageLevel,
    pub tuning: DeliveryTuning,
    pub transform: Option<Arc<dyn ghostr_delivery::transform::TransformBackend>>,
}

impl Default for DeliveryOptions {
    fn default() -> Self {
        Self {
            params: base_params(),
            level: DataUsageLevel::Balanced,
            tuning: test_tuning(),
            transform: None,
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

pub fn serial_long_retry_options(transient_attempts: u32) -> DeliveryOptions {
    let mut options = DeliveryOptions::default();
    options.params.balanced_concurrency = 1;
    options.tuning.retry.base = Duration::from_secs(5);
    options.tuning.retry.max = Duration::from_secs(5);
    options.tuning.retry.transient_attempts = transient_attempts;
    options
}

pub fn production_geometry_parallel_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams::default(),
        level: DataUsageLevel::Aggressive,
        ..DeliveryOptions::default()
    }
}

/// The production ladder, scaled down: pauses in milliseconds and a
/// revival window far longer than any test run.
fn test_tuning() -> DeliveryTuning {
    DeliveryTuning {
        probe_concurrency: 2,
        max_requests_per_authority: None,
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
