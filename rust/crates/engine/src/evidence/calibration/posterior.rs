use super::{CalibrationContext, CalibrationLabel, ReliabilityEstimate};

#[derive(Clone, Copy)]
pub(super) enum Level {
    Global,
    Issuer,
    Client,
    Origin,
    Url,
    Exact,
}

impl Level {
    pub(super) const ALL: [Self; 6] = [
        Self::Global,
        Self::Issuer,
        Self::Client,
        Self::Origin,
        Self::Url,
        Self::Exact,
    ];

    pub(super) fn matches(self, label: &CalibrationLabel, wanted: &CalibrationContext) -> bool {
        if label.context.field != wanted.field || label.context.context != wanted.context {
            return false;
        }
        let left = &label.context.dimensions;
        let right = &wanted.dimensions;
        match self {
            Self::Global => true,
            Self::Issuer => same(left.issuer.as_ref(), right.issuer.as_ref()),
            Self::Client => same(left.client.as_ref(), right.client.as_ref()),
            Self::Origin => same(left.origin.as_ref(), right.origin.as_ref()),
            Self::Url => same(left.url.as_ref(), right.url.as_ref()),
            Self::Exact => left == right,
        }
    }
}

fn same(left: Option<&String>, right: Option<&String>) -> bool {
    right.is_some_and(|value| left == Some(value))
}

#[derive(Clone, Copy)]
pub(super) struct Posterior {
    pub(super) alpha: f64,
    pub(super) beta: f64,
    pub(super) samples: f64,
}

impl Posterior {
    pub(super) fn uniform() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
            samples: 0.0,
        }
    }

    pub(super) fn mean(self) -> f64 {
        self.alpha / (self.alpha + self.beta).max(f64::EPSILON)
    }

    pub(super) fn estimate(self) -> ReliabilityEstimate {
        let total = (self.alpha + self.beta).max(f64::EPSILON);
        let mean = self.mean();
        let variance = self.alpha * self.beta / (total * total * (total + 1.0));
        let lower = (mean - 1.96 * variance.sqrt()).clamp(0.0, 1.0);
        ReliabilityEstimate {
            mean_bps: basis_points(mean),
            lower_bound_bps: basis_points(lower),
            effective_samples_bps: (self.samples * 10_000.0).round().min(u32::MAX as f64) as u32,
        }
    }
}

fn basis_points(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}
