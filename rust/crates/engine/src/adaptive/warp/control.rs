use super::ActionKind;
use crate::ActionId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationDecision {
    Continue,
    FinishBlockThenReplan,
    Abort,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContinuationPolicy {
    continue_hysteresis: i64,
    abort_hysteresis: i64,
}

impl ContinuationPolicy {
    pub const fn new(continue_hysteresis: i64, abort_hysteresis: i64) -> Self {
        Self {
            continue_hysteresis,
            abort_hysteresis,
        }
    }

    pub fn decide(self, continuation_advantage: i64) -> ContinuationDecision {
        if continuation_advantage > self.continue_hysteresis {
            return ContinuationDecision::Continue;
        }
        if continuation_advantage < -self.abort_hysteresis {
            return ContinuationDecision::Abort;
        }
        ContinuationDecision::FinishBlockThenReplan
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityProof {
    VerifiedHash([u8; 32]),
    IndependentWhole,
    Unverified,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HedgeInput {
    pub primary: ActionId,
    pub action: ActionKind,
    pub elapsed_ms: u64,
    pub tail_trigger_ms: u64,
    pub loss_reduction_micros: u64,
    pub duplicate_cost_micros: u64,
    pub urgent: bool,
}

impl HedgeInput {
    pub fn new(primary: ActionId, action: ActionKind) -> Self {
        Self {
            primary,
            action,
            elapsed_ms: 0,
            tail_trigger_ms: u64::MAX,
            loss_reduction_micros: 0,
            duplicate_cost_micros: u64::MAX,
            urgent: true,
        }
    }

    pub fn with_timing(mut self, elapsed_ms: u64, trigger_ms: u64) -> Self {
        self.elapsed_ms = elapsed_ms;
        self.tail_trigger_ms = trigger_ms;
        self
    }

    pub fn with_value(mut self, reduction_micros: u64, duplicate_cost_micros: u64) -> Self {
        self.loss_reduction_micros = reduction_micros;
        self.duplicate_cost_micros = duplicate_cost_micros;
        self
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HedgePolicy;

impl HedgePolicy {
    pub fn eligible(input: &HedgeInput, identity: IdentityProof) -> bool {
        input.urgent
            && input.elapsed_ms >= input.tail_trigger_ms
            && input.loss_reduction_micros > input.duplicate_cost_micros
            && small_action(&input.action)
            && identity_allows(&input.action, identity)
    }
}

fn small_action(action: &ActionKind) -> bool {
    match action {
        ActionKind::Prefix(bytes) | ActionKind::Tail(bytes) | ActionKind::FetchRange(bytes) => {
            bytes.len() <= 1024 * 1024
        }
        ActionKind::FetchWhole { maximum_bytes } => *maximum_bytes <= 1024 * 1024,
        _ => false,
    }
}

fn identity_allows(action: &ActionKind, identity: IdentityProof) -> bool {
    match action {
        ActionKind::FetchWhole { .. } => !matches!(identity, IdentityProof::Unverified),
        _ => matches!(identity, IdentityProof::VerifiedHash(_)),
    }
}
