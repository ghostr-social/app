use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct DiscountedBeta {
    positive: f64,
    negative: f64,
    last_at_ms: u64,
}

impl DiscountedBeta {
    pub fn observe(&mut self, positive: bool, at_ms: u64, half_life_ms: u64) {
        let (yes, no) = self.counts(at_ms, half_life_ms);
        self.positive = yes + f64::from(positive);
        self.negative = no + f64::from(!positive);
        self.last_at_ms = at_ms;
    }

    pub fn posterior(
        self,
        prior_alpha: f64,
        prior_beta: f64,
        at_ms: u64,
        half_life_ms: u64,
    ) -> BetaPosterior {
        let (positive, negative) = self.counts(at_ms, half_life_ms);
        BetaPosterior {
            alpha: prior_alpha + positive,
            beta: prior_beta + negative,
            evidence: positive + negative,
        }
    }

    fn counts(self, at_ms: u64, half_life_ms: u64) -> (f64, f64) {
        let age = at_ms.saturating_sub(self.last_at_ms);
        let weight = decay(age, half_life_ms);
        (self.positive * weight, self.negative * weight)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BetaPosterior {
    pub alpha: f64,
    pub beta: f64,
    pub evidence: f64,
}

impl BetaPosterior {
    pub fn mean(self) -> f64 {
        self.alpha / (self.alpha + self.beta).max(f64::EPSILON)
    }
}

pub(super) fn decay(age_ms: u64, half_life_ms: u64) -> f64 {
    0.5f64.powf(age_ms as f64 / half_life_ms.max(1) as f64)
}
