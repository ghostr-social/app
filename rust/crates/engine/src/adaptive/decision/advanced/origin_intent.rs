use crate::origin_model::OriginAdmissionIntent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RecordedOriginAdmissionIntent {
    #[default]
    Delivery,
    OptionalExploration,
}

impl RecordedOriginAdmissionIntent {
    pub(super) const fn is_delivery(&self) -> bool {
        matches!(self, Self::Delivery)
    }
}

impl From<OriginAdmissionIntent> for RecordedOriginAdmissionIntent {
    fn from(value: OriginAdmissionIntent) -> Self {
        match value {
            OriginAdmissionIntent::Delivery => Self::Delivery,
            OriginAdmissionIntent::OptionalExploration => Self::OptionalExploration,
        }
    }
}
