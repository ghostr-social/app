use super::super::warp::ResourceCost;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CompletionTimes {
    pub expected_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub cvar_ms: u64,
}

impl CompletionTimes {
    pub const fn new(expected_ms: u64, p95_ms: u64, p99_ms: u64, cvar_ms: u64) -> Self {
        Self {
            expected_ms,
            p95_ms,
            p99_ms,
            cvar_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeadlineReadiness {
    pub deadline_ms: u64,
    pub probability_bps: u16,
}

impl DeadlineReadiness {
    pub const fn new(deadline_ms: u64, probability_bps: u16) -> Self {
        Self {
            deadline_ms,
            probability_bps: if probability_bps > 10_000 {
                10_000
            } else {
                probability_bps
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QualityEstimate {
    pub expected_micros: u64,
    pub lower_micros: u64,
    pub uncertainty_bps: u16,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SizeBounds {
    pub lower: Option<u64>,
    pub upper: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanMetrics {
    pub readiness_bytes: u64,
    pub readiness_time: CompletionTimes,
    pub readiness_by_deadline: Vec<DeadlineReadiness>,
    pub ready_playback_ms: u64,
    pub ready_coverage_ms: u64,
    pub quality: QualityEstimate,
    pub resources: ResourceCost,
    pub unused_bytes: u64,
    pub storage_byte_ms: u64,
    pub streamability_bps: u16,
    pub integrity_bps: u16,
    pub size: SizeBounds,
    pub cache_value_micros: u64,
    pub information_value_micros: u64,
}

impl PlanMetrics {
    pub fn new(
        readiness_bytes: u64,
        readiness_time: CompletionTimes,
        readiness_by_deadline: Vec<DeadlineReadiness>,
        ready_playback_ms: u64,
    ) -> Self {
        Self {
            readiness_bytes,
            readiness_time,
            readiness_by_deadline,
            ready_playback_ms,
            ready_coverage_ms: ready_playback_ms,
            quality: QualityEstimate::default(),
            resources: ResourceCost::default(),
            unused_bytes: 0,
            storage_byte_ms: 0,
            streamability_bps: 0,
            integrity_bps: 0,
            size: SizeBounds::default(),
            cache_value_micros: 0,
            information_value_micros: 0,
        }
    }

    pub fn with_resources(mut self, resources: ResourceCost) -> Self {
        self.resources = resources;
        self
    }

    pub fn with_unused_bytes(mut self, bytes: u64) -> Self {
        self.unused_bytes = bytes;
        self
    }

    pub fn with_storage_byte_ms(mut self, value: u64) -> Self {
        self.storage_byte_ms = value;
        self
    }

    pub fn with_confidence(mut self, streamability_bps: u16, integrity_bps: u16) -> Self {
        self.streamability_bps = streamability_bps.min(10_000);
        self.integrity_bps = integrity_bps.min(10_000);
        self
    }

    pub fn with_quality(mut self, expected: u64, lower: u64, uncertainty_bps: u16) -> Self {
        self.quality = QualityEstimate {
            expected_micros: expected,
            lower_micros: lower.min(expected),
            uncertainty_bps: uncertainty_bps.min(10_000),
        };
        self
    }

    pub fn with_size_bounds(mut self, lower: Option<u64>, upper: Option<u64>) -> Self {
        self.size = SizeBounds { lower, upper };
        self
    }

    pub fn with_cache_value(mut self, value: u64) -> Self {
        self.cache_value_micros = value;
        self
    }

    pub fn with_information_value(mut self, value: u64) -> Self {
        self.information_value_micros = value;
        self
    }
}
