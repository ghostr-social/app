use super::{ResourceCost, ResourcePrices};
use crate::adaptive::CompletionTimes;
use crate::{ActionId, ByteRange, PostId, RequestAuthority};

mod conflict;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TransformKind {
    Remux,
    Segment,
    Transcode,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ActionKind {
    Head,
    Prefix(ByteRange),
    Tail(ByteRange),
    FetchRange(ByteRange),
    FetchWhole {
        maximum_bytes: u64,
    },
    Promote {
        active: ActionId,
        maximum_bytes: u64,
    },
    Transform(TransformKind),
    CacheUpgrade(ByteRange),
    Hedge {
        primary: ActionId,
        alternate: String,
    },
    Cancel(ActionId),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ActionValue {
    pub delay_loss_micros: i64,
    pub reserve_gain_micros: i64,
    pub information_value_micros: i64,
    pub exploration_micros: i64,
    pub cache_gain_micros: i64,
    pub tail_risk_micros: i64,
    pub cvar_micros: i64,
    pub rank_cost_micros: i64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ActionForecast {
    pub completion: CompletionTimes,
    pub success_bps: u16,
    pub ready_playback_ms: u64,
    pub quality_gain_micros: u64,
    pub cache_reuse_bps: u16,
}

impl ActionForecast {
    pub const fn new(
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

    pub const fn with_quality(mut self, gain_micros: u64) -> Self {
        self.quality_gain_micros = gain_micros;
        self
    }

    pub const fn with_cache_reuse(mut self, probability_bps: u16) -> Self {
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

    pub fn total(self, resources: ResourceCost, prices: ResourcePrices) -> i64 {
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
    pub value: ActionValue,
    pub resources: ResourceCost,
    pub forecast: ActionForecast,
    pub(super) origin: String,
    request_authority: Option<RequestAuthority>,
    pub requires: Vec<u16>,
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
            origin: String::new(),
            request_authority: None,
            requires: Vec::new(),
        }
    }

    pub fn with_resources(mut self, resources: ResourceCost) -> Self {
        self.resources = resources;
        self
    }

    pub fn with_forecast(mut self, forecast: ActionForecast) -> Self {
        self.forecast = forecast;
        self
    }

    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        let origin = origin.into();
        self.request_authority = RequestAuthority::from_url(&origin);
        self.origin = origin;
        self
    }

    pub fn request_authority(&self) -> Option<&RequestAuthority> {
        self.request_authority.as_ref()
    }

    pub fn requiring(mut self, requirements: &[u16]) -> Self {
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
