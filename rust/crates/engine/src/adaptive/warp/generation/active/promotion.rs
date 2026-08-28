use super::super::builder::{Builder, NodeInput};
use super::super::{GeneratedAction, PlannerCommand, PromotionGenerationPolicy};
use crate::adaptive::{
    ActionKind, CandidateSnapshot, InFlightAction, PromotionGrant, ResourceCost,
};

impl Builder<'_> {
    pub(super) fn add_promotion(&mut self, candidate: &CandidateSnapshot, active: &InFlightAction) {
        let Some((grant, delta)) =
            target(active, self.snapshot.observed_at_ms, self.promotion_policy)
        else {
            return;
        };
        let forecast_kind = ActionKind::FetchWhole {
            maximum_bytes: grant.maximum_bytes,
        };
        let prediction = prediction(self, candidate, active, &forecast_kind, grant.maximum_bytes);
        let kind = ActionKind::Promote {
            active: active.action_id,
            maximum_bytes: grant.maximum_bytes,
        };
        let input = NodeInput::new(kind, &active.source, prediction, &[]);
        let mut node = resources(
            self.node(candidate, input),
            grant.maximum_bytes,
            delta,
            self.promotion_policy,
        );
        if self.promotion_policy == PromotionGenerationPolicy::LegacyLatentGrant {
            node.value.cache_gain_micros =
                super::super::value::legacy_promotion_cache_gain(candidate);
        }
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::Promote {
                post: candidate.post.clone(),
                action: active.action_id,
                source: active.source.clone(),
                grant,
            },
        });
    }
}

fn prediction(
    builder: &Builder<'_>,
    candidate: &CandidateSnapshot,
    active: &InFlightAction,
    kind: &ActionKind,
    unread_body_bytes: u64,
) -> super::super::prediction::Prediction {
    match builder.promotion_policy {
        PromotionGenerationPolicy::LegacyLatentGrant => {
            builder.prediction(candidate, kind, &active.source)
        }
        PromotionGenerationPolicy::ObservedResponse => {
            let opportunity = active
                .promotion_opportunity
                .expect("observed response promotion requires its opportunity");
            let target = super::super::continuation::Target::new(
                kind,
                &active.source,
                unread_body_bytes,
                opportunity.request_profile(unread_body_bytes),
            );
            super::super::continuation::predict(builder, candidate, target)
        }
    }
}

fn resources(
    mut node: crate::adaptive::ActionNode,
    unread_body_bytes: u64,
    delta: u64,
    policy: PromotionGenerationPolicy,
) -> crate::adaptive::ActionNode {
    let authority = ResourceCost::new(delta, delta, 0, 0);
    if policy == PromotionGenerationPolicy::LegacyLatentGrant {
        node.resources = authority;
        return node;
    }
    node.resources = ResourceCost::new(unread_body_bytes, unread_body_bytes, 0, 0);
    node.with_resource_authority(authority)
}

fn target(
    active: &InFlightAction,
    observed_at_ms: u64,
    policy: PromotionGenerationPolicy,
) -> Option<(PromotionGrant, u64)> {
    let latent = active.request.promotion()?;
    let maximum_bytes = match policy {
        PromotionGenerationPolicy::LegacyLatentGrant => latent.maximum_bytes,
        PromotionGenerationPolicy::ObservedResponse => {
            active.promotion_opportunity?.contract().maximum_bytes()
        }
    };
    let grant = PromotionGrant {
        maximum_bytes,
        valid_until_ms: latent.valid_until_ms,
    };
    if active.cancelling || !active.identity_current || grant.valid_until_ms < observed_at_ms {
        return None;
    }
    let delta = grant
        .maximum_bytes
        .checked_sub(active.reserved_storage_bytes)?;
    let positive = policy == PromotionGenerationPolicy::ObservedResponse || delta > 0;
    (grant.maximum_bytes <= latent.maximum_bytes && positive).then_some((grant, delta))
}
