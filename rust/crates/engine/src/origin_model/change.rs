use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct ChangeDetector {
    surprise_streak: u8,
    changed_at_ms: Option<u64>,
}

impl ChangeDetector {
    pub fn observe(&mut self, surprise: f64, at_ms: u64) -> bool {
        self.surprise_streak = match surprise >= 0.5 {
            true => self.surprise_streak.saturating_add(1),
            false => self.surprise_streak.saturating_sub(1),
        };
        if self.surprise_streak < 3 {
            return false;
        }
        self.surprise_streak = 0;
        self.changed_at_ms = Some(at_ms);
        true
    }

    pub fn short_weight(self, at_ms: u64, adaptation_ms: u64) -> f64 {
        let Some(changed) = self.changed_at_ms else {
            return 0.15;
        };
        let age = at_ms.saturating_sub(changed);
        if age >= adaptation_ms {
            return 0.15;
        }
        0.15 + 0.80 * (1.0 - age as f64 / adaptation_ms.max(1) as f64)
    }

    pub fn adapting(self, at_ms: u64, adaptation_ms: u64) -> bool {
        self.changed_at_ms
            .is_some_and(|changed| at_ms.saturating_sub(changed) < adaptation_ms)
    }
}
