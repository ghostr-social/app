use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const WINDOW_MS: u64 = 60_000;
const GLOBAL_LIMIT: u8 = 4;
const ORIGIN_LIMIT: u8 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplorationClaim {
    origin: String,
    window_started_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct ExplorationBudget {
    window_started_ms: u64,
    global_claims: u8,
    per_origin: BTreeMap<String, u8>,
}

impl ExplorationBudget {
    pub fn can_claim(&self, origin: &str, at_ms: u64) -> bool {
        if self.window_expired(at_ms) {
            return true;
        }
        let origin_claims = self.per_origin.get(origin).copied().unwrap_or_default();
        self.global_claims < GLOBAL_LIMIT && origin_claims < ORIGIN_LIMIT
    }

    pub fn claim(&mut self, origin: &str, at_ms: u64) -> Option<ExplorationClaim> {
        self.refresh(at_ms);
        if !self.can_claim(origin, at_ms) {
            return None;
        }
        let origin_claims = self.per_origin.get(origin).copied().unwrap_or_default();
        self.global_claims += 1;
        self.per_origin.insert(origin.to_owned(), origin_claims + 1);
        Some(ExplorationClaim {
            origin: origin.to_owned(),
            window_started_ms: self.window_started_ms,
        })
    }

    pub fn release(&mut self, claim: &ExplorationClaim) {
        if claim.window_started_ms != self.window_started_ms {
            return;
        }
        let Some(count) = self.per_origin.get_mut(&claim.origin) else {
            return;
        };
        *count = count.saturating_sub(1);
        self.global_claims = self.global_claims.saturating_sub(1);
        if *count == 0 {
            self.per_origin.remove(&claim.origin);
        }
    }

    fn refresh(&mut self, at_ms: u64) {
        if self.window_started_ms == 0 {
            self.window_started_ms = at_ms;
            return;
        }
        if !self.window_expired(at_ms) {
            return;
        }
        self.window_started_ms = at_ms;
        self.global_claims = 0;
        self.per_origin.clear();
    }

    fn window_expired(&self, at_ms: u64) -> bool {
        self.window_started_ms != 0 && at_ms.saturating_sub(self.window_started_ms) >= WINDOW_MS
    }

    pub(super) fn replay_project(&self, aliases: impl Fn(&str) -> Vec<String>) -> Self {
        let per_origin = self
            .per_origin
            .iter()
            .flat_map(|(origin, claims)| {
                aliases(origin)
                    .into_iter()
                    .map(|projected| (projected, *claims))
            })
            .collect();
        Self {
            window_started_ms: self.window_started_ms,
            global_claims: self.global_claims,
            per_origin,
        }
    }

    pub(super) fn replay_bounded(&self) -> bool {
        self.per_origin.len() <= usize::from(GLOBAL_LIMIT)
    }
}
