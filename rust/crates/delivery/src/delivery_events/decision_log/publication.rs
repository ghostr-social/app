use ghostr_engine::adaptive::{
    AllocationPlan, DecisionModelInput, DecisionPrivacy, DecisionRecord, DecisionRecordInput,
    PlayabilitySnapshot, ShadowPrices, WarpDecisionRecordInput, WarpPlanningDecision,
};

pub(crate) struct LegacyDecisionPublication<'a> {
    pub snapshot: &'a PlayabilitySnapshot,
    pub plan: &'a AllocationPlan,
    pub prices: ShadowPrices,
    pub models: &'a [DecisionModelInput],
}

pub(crate) struct WarpDecisionPublication<'a> {
    pub snapshot: &'a PlayabilitySnapshot,
    pub decision: &'a WarpPlanningDecision,
    pub legacy_prices: ShadowPrices,
    pub models: &'a [DecisionModelInput],
}

pub(super) enum DecisionPublication<'a> {
    Legacy(LegacyDecisionPublication<'a>),
    Warp(WarpDecisionPublication<'a>),
}

impl DecisionPublication<'_> {
    pub(super) fn capture(self, sequence: u64, privacy: &DecisionPrivacy) -> DecisionRecord {
        match self {
            Self::Legacy(value) => DecisionRecord::capture(DecisionRecordInput {
                sequence,
                snapshot: value.snapshot,
                allocation: value.plan,
                shadow_prices: value.prices,
                models: value.models,
                privacy,
            }),
            Self::Warp(value) => DecisionRecord::capture_warp(WarpDecisionRecordInput {
                sequence,
                snapshot: value.snapshot,
                decision: value.decision,
                legacy_shadow_prices: value.legacy_prices,
                models: value.models,
                privacy,
            }),
        }
    }
}

impl<'a> From<LegacyDecisionPublication<'a>> for DecisionPublication<'a> {
    fn from(value: LegacyDecisionPublication<'a>) -> Self {
        Self::Legacy(value)
    }
}

impl<'a> From<WarpDecisionPublication<'a>> for DecisionPublication<'a> {
    fn from(value: WarpDecisionPublication<'a>) -> Self {
        Self::Warp(value)
    }
}
