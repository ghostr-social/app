use super::*;

#[cfg(test)]
impl ProbabilityMass {
    fn shifted(&self, offset_ms: u64) -> Self {
        Self::normalize(
            self.0
                .iter()
                .map(|point| Point {
                    at_ms: point.at_ms.saturating_add(offset_ms),
                    probability: point.probability,
                })
                .collect(),
        )
    }
}

impl WatchDistribution {
    pub fn p50_ms(&self) -> u64 {
        self.0.quantile(0.50)
    }
}

impl DeadlineDistribution {
    #[cfg(test)]
    pub(crate) fn shifted(&self, offset_ms: u64) -> Self {
        Self(self.0.shifted(offset_ms))
    }
}
