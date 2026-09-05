use super::WarpPlannerConfig;

impl WarpPlannerConfig {
    pub(crate) fn with_lookahead(mut self) -> Self {
        self.profile = super::PlannerProfile::Lookahead1;
        self
    }

    pub(crate) const fn with_rescue_thresholds(
        mut self,
        safety_bps: u16,
        emergency_bps: u16,
    ) -> Self {
        self.safety_rescue_bps = clamp_bps(safety_bps);
        self.emergency_rescue_bps = clamp_bps(emergency_bps);
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
