use crate::adaptive::FeedOffset;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NavigationPreparationPolicy {
    #[default]
    ForwardOnly,
    Bidirectional,
    BidirectionalWindow,
}

impl NavigationPreparationPolicy {
    pub(crate) fn is_legacy(&self) -> bool {
        *self == Self::ForwardOnly
    }

    pub(crate) fn includes_knowledge(self, offset: FeedOffset) -> bool {
        match self {
            Self::ForwardOnly | Self::Bidirectional => (1..=8).contains(&offset.value()),
            Self::BidirectionalWindow => (-5..=24).contains(&offset.value()),
        }
    }

    pub(crate) fn includes_payload(self, offset: FeedOffset) -> bool {
        self.includes_depth(offset, 2)
    }

    pub(crate) fn includes_depth(self, offset: FeedOffset, depth: u32) -> bool {
        match self {
            Self::ForwardOnly => offset.value() >= 0 && offset.magnitude() <= depth,
            Self::Bidirectional | Self::BidirectionalWindow => offset.magnitude() <= depth,
        }
    }
}
