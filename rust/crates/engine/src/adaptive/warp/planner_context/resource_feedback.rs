use crate::adaptive::{ResourceObservation, ResourcePrices};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceFeedback {
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub revision: u64,
    pub actual: ResourceObservation,
    pub target: ResourceObservation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_snapshot: Option<ResourcePriceSnapshot>,
}

impl ResourceFeedback {
    pub const fn authoritative(
        price_snapshot: ResourcePriceSnapshot,
        actual: ResourceObservation,
        target: ResourceObservation,
    ) -> Self {
        Self {
            revision: price_snapshot.cursor.revision,
            actual,
            target,
            price_snapshot: Some(price_snapshot),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ResourceFeedbackCursor {
    pub epoch: u64,
    pub revision: u64,
}

impl ResourceFeedbackCursor {
    pub const fn new(epoch: u64, revision: u64) -> Self {
        Self { epoch, revision }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourcePriceSnapshot {
    pub cursor: ResourceFeedbackCursor,
    pub prices: ResourcePrices,
}

impl ResourcePriceSnapshot {
    pub const fn new(cursor: ResourceFeedbackCursor, prices: ResourcePrices) -> Self {
        Self { cursor, prices }
    }
}

const fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}
