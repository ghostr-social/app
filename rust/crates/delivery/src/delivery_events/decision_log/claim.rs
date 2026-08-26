use super::lifecycle::{unix_time_ms, with_elapsed};
use super::{retention, trim, DecisionClaim, DecisionLog, DecisionResolution, DecisionToken};
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::representation::TransferIdentity;
use std::sync::Arc;

#[derive(Clone, Copy)]
enum AbandonedCarrier {
    Token,
    Claim { elapsed_ms: u64 },
}

impl DecisionLog {
    pub(super) fn claim(
        &self,
        mut token: DecisionToken,
        identity: &TransferIdentity,
        started_at_ms: u64,
    ) -> Result<DecisionClaim, DecisionToken> {
        if !token.belongs_to(&self.store) {
            return Err(token);
        }
        let mut store = self.lock();
        if !claimable(&store, token.sequence, identity, &self.privacy)
            || !store.claimed.insert(token.sequence)
        {
            return Err(token);
        }
        let claim = DecisionClaim::new(&token, started_at_ms);
        token.armed = false;
        Ok(claim)
    }

    pub(super) fn resolve_claim(
        &self,
        mut claim: DecisionClaim,
        outcome: DecisionOutcome,
        observed_at_ms: u64,
    ) -> Option<DecisionResolution> {
        if outcome == DecisionOutcome::Pending || !claim.belongs_to(&self.store) {
            return None;
        }
        let mut store = self.lock();
        let elapsed_ms = observed_at_ms.saturating_sub(claim.started_at_ms);
        let outcome = with_elapsed(outcome, elapsed_ms);
        let claimed = store.claimed.contains(&claim.sequence);
        let resolved =
            retention::resolve_claimed(&mut store.records, claimed, claim.sequence, outcome)?;
        complete_claim(&mut store, claim.sequence);
        claim.armed = false;
        Some(DecisionResolution {
            action: resolved.0,
            warp_action: resolved.1,
            elapsed_ms,
        })
    }
}

impl DecisionClaim {
    fn new(token: &DecisionToken, started_at_ms: u64) -> Self {
        Self {
            sequence: token.sequence,
            owner: std::sync::Weak::clone(&token.owner),
            started_at_ms,
            armed: true,
        }
    }

    fn belongs_to(&self, store: &Arc<std::sync::Mutex<super::DecisionStore>>) -> bool {
        std::sync::Weak::ptr_eq(&self.owner, &Arc::downgrade(store))
    }
}

impl Drop for DecisionToken {
    fn drop(&mut self) {
        abandon(
            self.armed,
            &self.owner,
            self.sequence,
            AbandonedCarrier::Token,
        );
    }
}

impl Drop for DecisionClaim {
    fn drop(&mut self) {
        let elapsed_ms = unix_time_ms().saturating_sub(self.started_at_ms);
        abandon(
            self.armed,
            &self.owner,
            self.sequence,
            AbandonedCarrier::Claim { elapsed_ms },
        );
    }
}

fn claimable(
    store: &super::DecisionStore,
    sequence: u64,
    identity: &TransferIdentity,
    privacy: &ghostr_engine::adaptive::DecisionPrivacy,
) -> bool {
    let Some(record) = store
        .records
        .iter()
        .find(|record| record.sequence == sequence)
    else {
        return false;
    };
    record.eventual_outcome == DecisionOutcome::Pending
        && record.chosen_action_id.is_none()
        && record.authorizes_probe_claim(identity, privacy)
}

fn complete_claim(store: &mut super::DecisionStore, sequence: u64) {
    store.claimed.remove(&sequence);
    store.completed.push_back(sequence);
    trim(store);
}

fn abandon(
    armed: bool,
    owner: &std::sync::Weak<std::sync::Mutex<super::DecisionStore>>,
    sequence: u64,
    carrier: AbandonedCarrier,
) {
    if !armed {
        return;
    }
    let Some(owner) = owner.upgrade() else {
        return;
    };
    let mut store = owner
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    resolve_abandoned(&mut store, sequence, carrier);
}

fn resolve_abandoned(store: &mut super::DecisionStore, sequence: u64, carrier: AbandonedCarrier) {
    let outcome = carrier.failure_outcome();
    if !carrier.resolve(store, sequence, outcome) {
        return;
    }
    carrier.remove_claim(store, sequence);
    store.completed.push_back(sequence);
    trim(store);
}

impl AbandonedCarrier {
    fn failure_outcome(self) -> DecisionOutcome {
        match self {
            Self::Token => DecisionOutcome::Failed {
                class: "decision_token_abandoned".into(),
                elapsed_ms: 0,
            },
            Self::Claim { elapsed_ms } => DecisionOutcome::Failed {
                class: "warp_head_probe_abandoned".into(),
                elapsed_ms,
            },
        }
    }

    fn resolve(
        self,
        store: &mut super::DecisionStore,
        sequence: u64,
        outcome: DecisionOutcome,
    ) -> bool {
        let claimed = store.claimed.contains(&sequence);
        match self {
            Self::Token => {
                retention::resolve_unbound(&mut store.records, claimed, sequence, outcome).is_some()
            }
            Self::Claim { .. } => {
                retention::resolve_claimed(&mut store.records, claimed, sequence, outcome).is_some()
            }
        }
    }

    fn remove_claim(self, store: &mut super::DecisionStore, sequence: u64) {
        if matches!(self, Self::Claim { .. }) {
            store.claimed.remove(&sequence);
        }
    }
}
