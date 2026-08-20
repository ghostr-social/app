use serde::{Deserialize, Serialize};

pub const CONFIDENCE_SCALE: u16 = 10_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(u16);

impl Confidence {
    pub fn new(basis_points: u16) -> Option<Self> {
        (basis_points <= CONFIDENCE_SCALE).then_some(Self(basis_points))
    }

    pub const fn none() -> Self {
        Self(0)
    }

    pub const fn certain() -> Self {
        Self(CONFIDENCE_SCALE)
    }

    pub const fn basis_points(self) -> u16 {
        self.0
    }

    pub(crate) fn with_agreement(self, count: usize) -> Self {
        let bonus = count.saturating_sub(1).min(4) as u16 * 250;
        Self(self.0.saturating_add(bonus).min(CONFIDENCE_SCALE))
    }

    pub(crate) fn decayed(self, age_ms: u64, half_life_ms: u64) -> Self {
        let factor = 0.5_f64.powf(age_ms as f64 / half_life_ms.max(1) as f64);
        Self((f64::from(self.0) * factor).round() as u16)
    }
}
