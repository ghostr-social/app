use crate::adaptive::ActionValue;
use crate::origin_model::OriginAdmissionIntent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HlsGenerationPolicy {
    LegacyWholeStage,
    BoundedObjectCursor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PromotionGenerationPolicy {
    LegacyLatentGrant,
    ObservedResponse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RangeAliasGenerationPolicy {
    LegacyIndependentActions,
    PromotableDominance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OriginAdmissionGenerationPolicy {
    LegacyUnclassified,
    TypedIntent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WarpGenerationPolicies {
    pub(crate) hls: HlsGenerationPolicy,
    pub(crate) promotion: PromotionGenerationPolicy,
    pub(crate) range_alias: RangeAliasGenerationPolicy,
    pub(crate) origin_admission: OriginAdmissionGenerationPolicy,
}

impl WarpGenerationPolicies {
    pub(crate) const fn current() -> Self {
        Self {
            hls: HlsGenerationPolicy::BoundedObjectCursor,
            promotion: PromotionGenerationPolicy::ObservedResponse,
            range_alias: RangeAliasGenerationPolicy::PromotableDominance,
            origin_admission: OriginAdmissionGenerationPolicy::TypedIntent,
        }
    }

    pub(crate) const fn apply_origin(
        self,
        value: ActionValue,
        intent: OriginAdmissionIntent,
    ) -> (ActionValue, OriginAdmissionIntent) {
        match self.origin_admission {
            OriginAdmissionGenerationPolicy::LegacyUnclassified => {
                (value, OriginAdmissionIntent::Delivery)
            }
            OriginAdmissionGenerationPolicy::TypedIntent => {
                (value.for_origin_intent(intent), intent)
            }
        }
    }
}
