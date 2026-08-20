use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const WINDOW_MS: u64 = 60_000;
const GLOBAL_LIMIT: u8 = 4;
const ORIGIN_LIMIT: u8 = 1;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct ExplorationBudget {
    window_started_ms: u64,
    global_claims: u8,
    per_origin: BTreeMap<String, u8>,
}

impl ExplorationBudget {
    pub fn claim(&mut self, origin: &str, at_ms: u64) -> bool {
        self.refresh(at_ms);
        let origin_claims = self.per_origin.get(origin).copied().unwrap_or_default();
        if self.global_claims >= GLOBAL_LIMIT || origin_claims >= ORIGIN_LIMIT {
            return false;
        }
        self.global_claims += 1;
        self.per_origin.insert(origin.to_owned(), origin_claims + 1);
        true
    }

    fn refresh(&mut self, at_ms: u64) {
        if self.window_started_ms == 0 {
            self.window_started_ms = at_ms;
            return;
        }
        if at_ms.saturating_sub(self.window_started_ms) < WINDOW_MS {
            return;
        }
        self.window_started_ms = at_ms;
        self.global_claims = 0;
        self.per_origin.clear();
    }
}
