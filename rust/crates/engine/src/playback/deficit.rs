//! Finite-horizon conservation of contiguous, dependency-complete media.
//! Results are conditional on the supplied trajectory, not Internet guarantees.
use super::PlaybackPhase;

const MAX_ARRIVALS: usize = 4_096;
const MAX_HORIZON_MS: u64 = 120_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UsableArrival {
    at_ms: u64,
    contiguous_ms: u64,
}

impl UsableArrival {
    /// `contiguous_ms` is the cumulative newly usable frontier beyond the
    /// initial buffer, after all required tracks and processing complete.
    pub(crate) const fn new(at_ms: u64, contiguous_ms: u64) -> Self {
        Self {
            at_ms,
            contiguous_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScenarioError {
    ResourceLimit,
    UnorderedArrivals,
    RegressingCoverage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BufferScenario {
    horizon_ms: u64,
    rate_milli: u16,
}

impl BufferScenario {
    pub(crate) const fn new(horizon_ms: u64, rate_milli: u16, phase: PlaybackPhase) -> Self {
        let consumes = matches!(
            phase,
            PlaybackPhase::Starting | PlaybackPhase::Playing | PlaybackPhase::NetworkStalled
        );
        Self {
            horizon_ms,
            rate_milli: if consumes { rate_milli } else { 0 },
        }
    }

    /// Computes pre-arrival left limits, retaining fractional consumption
    /// until the final upward rounding. Repeated frontiers are counted once.
    ///
    /// # Errors
    /// Rejects excessive work, unordered times, and regressing frontiers.
    pub(crate) fn required_ms(self, arrivals: &[UsableArrival]) -> Result<u64, ScenarioError> {
        self.validate(arrivals)?;
        let mut available = 0;
        let mut required = 0;
        for arrival in arrivals
            .iter()
            .take_while(|item| item.at_ms <= self.horizon_ms)
        {
            required = required.max(self.deficit(arrival.at_ms, available));
            available = u128::from(arrival.contiguous_ms) * 1_000;
        }
        required = required.max(self.deficit(self.horizon_ms, available));
        Ok(rounded_ms(required))
    }

    fn deficit(self, at_ms: u64, available: u128) -> u128 {
        (u128::from(at_ms) * u128::from(self.rate_milli)).saturating_sub(available)
    }

    fn validate(self, arrivals: &[UsableArrival]) -> Result<(), ScenarioError> {
        if arrivals.len() > MAX_ARRIVALS || self.horizon_ms > MAX_HORIZON_MS {
            return Err(ScenarioError::ResourceLimit);
        }
        for pair in arrivals.windows(2) {
            validate_successor(pair[0], pair[1])?;
        }
        Ok(())
    }
}

fn validate_successor(previous: UsableArrival, next: UsableArrival) -> Result<(), ScenarioError> {
    if previous.at_ms > next.at_ms {
        return Err(ScenarioError::UnorderedArrivals);
    }
    if previous.contiguous_ms > next.contiguous_ms {
        return Err(ScenarioError::RegressingCoverage);
    }
    Ok(())
}

fn rounded_ms(value: u128) -> u64 {
    value.div_ceil(1_000).min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
#[path = "deficit/test_support.rs"]
mod test_support;
