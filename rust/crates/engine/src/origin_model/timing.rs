#[derive(Clone, Copy)]
pub(super) struct ModelTiming {
    pub long_ms: u64,
    pub short_ms: u64,
    pub adaptation_ms: u64,
}

impl Default for ModelTiming {
    fn default() -> Self {
        Self {
            long_ms: 21_600_000,
            short_ms: 120_000,
            adaptation_ms: 1_800_000,
        }
    }
}
