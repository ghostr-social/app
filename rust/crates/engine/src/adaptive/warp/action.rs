use super::{ResourceCost, ResourcePrices};
use crate::adaptive::{CompletionTimes, RetrievalRequest};
use crate::{PostId, RequestAuthority};

mod conflict;
mod kind;
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

    pub(crate) const fn with_quality(mut self, gain_micros: u64) -> Self {
        self.quality_gain_micros = gain_micros;
        self
    }

    pub(crate) const fn with_cache_reuse(mut self, probability_bps: u16) -> Self {
        self.cache_reuse_bps = clamp_bps(probability_bps);
        self
    }
}

impl Default for ActionForecast {
    fn default() -> Self {
        Self::new(CompletionTimes::new(0, 0, 0, 0), 10_000, 0)
    }
}

impl ActionValue {
    pub const fn from_net_micros(value: i64) -> Self {
        Self {
            delay_loss_micros: value,
            reserve_gain_micros: 0,
            information_value_micros: 0,
            exploration_micros: 0,
            cache_gain_micros: 0,
            tail_risk_micros: 0,
            cvar_micros: 0,
            rank_cost_micros: 0,
        }
    }

    pub(crate) fn total(self, resources: ResourceCost, prices: ResourcePrices) -> i64 {
        let benefits = self
            .delay_loss_micros
            .saturating_add(self.reserve_gain_micros)
            .saturating_add(self.information_value_micros)
            .saturating_add(self.exploration_micros)
            .saturating_add(self.cache_gain_micros);
        benefits
            .saturating_sub(prices.cost(resources))
            .saturating_sub(self.tail_risk_micros)
            .saturating_sub(self.cvar_micros)
            .saturating_sub(self.rank_cost_micros)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ActionNode {
    pub id: u16,
    pub post: PostId,
    pub kind: ActionKind,
    pub(crate) value: ActionValue,
    pub resources: ResourceCost,
    pub forecast: ActionForecast,
    request_profile: Option<crate::origin_model::OriginRequestProfile>,
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
            forecast: ActionForecast::default(),
            request_profile: None,
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
        let mut authorized = self.resources;
        if let ActionKind::HlsBootstrap { maximum_bytes, .. } = &self.kind {
            authorized.network_bytes = *maximum_bytes;
        }
        authorized
    }

    pub(crate) fn with_forecast(mut self, forecast: ActionForecast) -> Self {
        self.forecast = forecast;
        self
    }

    pub(crate) fn with_request_profile(
        mut self,
        profile: Option<crate::origin_model::OriginRequestProfile>,
    ) -> Self {
        self.request_profile = profile;
        self
    }

    pub const fn request_profile(&self) -> Option<crate::origin_model::OriginRequestProfile> {
        self.request_profile
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
