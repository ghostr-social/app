use super::ResourceCost;
use crate::adaptive::{CompletionTimes, RetrievalRequest};
use crate::{PostId, RequestAuthority};

mod conflict;
mod kind;
mod value;
pub use kind::{ActionKind, TransformKind};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ActionValue {
    pub(crate) delay_loss_micros: i64,
    pub(crate) reserve_gain_micros: i64,
    pub(crate) information_value_micros: i64,
    pub(crate) exploration_micros: i64,
    pub(crate) cache_gain_micros: i64,
    pub(crate) tail_risk_micros: i64,
    pub(crate) cvar_micros: i64,
    pub(crate) rank_cost_micros: i64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ActionForecast {
    pub(crate) completion: CompletionTimes,
    pub(crate) success_bps: u16,
    pub ready_playback_ms: u64,
    pub quality_gain_micros: u64,
    pub(crate) cache_reuse_bps: u16,
}

impl ActionForecast {
    pub(crate) const fn new(
        completion: CompletionTimes,
        success_bps: u16,
        ready_playback_ms: u64,
    ) -> Self {
        Self {
            completion,
            success_bps: clamp_bps(success_bps),
            ready_playback_ms,
            quality_gain_micros: 0,
            cache_reuse_bps: 0,
        }
    }
}

impl Default for ActionForecast {
    fn default() -> Self {
        Self::new(CompletionTimes::new(0, 0, 0, 0), 10_000, 0)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ActionNode {
    pub id: u16,
    pub post: PostId,
    pub kind: ActionKind,
    pub(crate) value: ActionValue,
    pub resources: ResourceCost,
    resource_authority: Option<ResourceCost>,
    pub forecast: ActionForecast,
    request_profile: Option<crate::origin_model::OriginRequestProfile>,
    origin_admission_intent: crate::origin_model::OriginAdmissionIntent,
    pub(super) origin: String,
    request_authority: Option<RequestAuthority>,
    pub(crate) requires: Vec<u16>,
    request: Option<RetrievalRequest>,
}

impl ActionNode {
    pub fn new(id: u16, post: PostId, kind: ActionKind, value: ActionValue) -> Self {
        Self {
            id,
            post,
            kind,
            value,
            resources: ResourceCost::default(),
            resource_authority: None,
            forecast: ActionForecast::default(),
            request_profile: None,
            origin_admission_intent: crate::origin_model::OriginAdmissionIntent::Delivery,
            origin: String::new(),
            request_authority: None,
            requires: Vec::new(),
            request: None,
        }
    }

    pub fn with_resources(mut self, resources: ResourceCost) -> Self {
        self.resources = resources;
        self
    }

    pub fn authorized_resources(&self) -> ResourceCost {
        let mut authorized = self.resource_authority.unwrap_or(self.resources);
        if let ActionKind::HlsBootstrap { maximum_bytes, .. } = &self.kind {
            authorized.network_bytes = *maximum_bytes;
        }
        authorized
    }

    pub(super) const fn with_resource_authority(mut self, resources: ResourceCost) -> Self {
        self.resource_authority = Some(resources);
        self
    }

    pub(crate) const fn resource_authority(&self) -> Option<ResourceCost> {
        self.resource_authority
    }

    pub(crate) fn with_forecast(mut self, forecast: ActionForecast) -> Self {
        self.forecast = forecast;
        self
    }

    pub(super) fn with_request_profile(
        mut self,
        profile: Option<crate::origin_model::OriginRequestProfile>,
    ) -> Self {
        self.request_profile = profile;
        self
    }

    pub const fn request_profile(&self) -> Option<crate::origin_model::OriginRequestProfile> {
        self.request_profile
    }

    pub(super) const fn with_origin_admission_intent(
        mut self,
        intent: crate::origin_model::OriginAdmissionIntent,
    ) -> Self {
        self.origin_admission_intent = intent;
        self
    }

    pub const fn origin_admission_intent(&self) -> crate::origin_model::OriginAdmissionIntent {
        self.origin_admission_intent
    }

    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        let origin = origin.into();
        self.request_authority = RequestAuthority::from_url(&origin);
        self.origin = origin;
        self
    }

    pub(super) fn with_request(mut self, request: RetrievalRequest) -> Self {
        self.request = Some(request);
        self
    }

    pub(crate) fn request_authority(&self) -> Option<&RequestAuthority> {
        self.request_authority.as_ref()
    }

    pub(crate) fn replay_origin(&self) -> &str {
        &self.origin
    }

    pub(in crate::adaptive::warp) const fn request(&self) -> Option<RetrievalRequest> {
        self.request
    }

    pub(crate) fn requiring(mut self, requirements: &[u16]) -> Self {
        self.requires = requirements.to_vec();
        self
    }
}

const fn clamp_bps(value: u16) -> u16 {
    if value > 10_000 {
        10_000
    } else {
        value
    }
}

#[cfg(test)]
#[path = "action_axiom_test.rs"]
mod axiom_test_support;
