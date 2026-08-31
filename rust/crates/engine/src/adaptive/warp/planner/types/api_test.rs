use super::{ReserveProgressPolicy, WarpPlannerConfig};

impl WarpPlannerConfig {
    pub(crate) const fn with_rescue_thresholds(
        mut self,
        safety_bps: u16,
        emergency_bps: u16,
    ) -> Self {
        self.safety_rescue_bps = clamp_bps(safety_bps);
        self.emergency_rescue_bps = clamp_bps(emergency_bps);
        self
    }

    pub(crate) const fn with_legacy_reserve_progress_for_test(mut self) -> Self {
        self.reserve_progress_policy = ReserveProgressPolicy::LegacyCoverage;
        self
    }
}

const fn clamp_bps(value: u16) -> u16 {
    if value > 10_000 {
        10_000
    } else {
        value
    }
}
