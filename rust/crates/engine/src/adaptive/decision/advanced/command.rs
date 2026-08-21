use super::super::privacy::DecisionPrivacy;
use crate::adaptive::{PlannerCommand, TransformKind};
use serde::{Deserialize, Serialize};

mod allocation;
mod request;
pub use allocation::{
    RecordedAllocationReason, RecordedCandidateUtility, RecordedPreemptionAuthority,
    RecordedTransfer,
};
pub use request::{
    RecordedPromotionGrant, RecordedRetrievalRequest, RecordedWholeBodyContract,
    RecordedWholeFetchReason,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum RecordedWarpCommand {
    ProbeHead {
        post_id: String,
        source_id: String,
    },
    Transfer {
        transfer: RecordedTransfer,
    },
    Promote {
        post_id: String,
        action_id: u64,
    },
    Transform {
        post_id: String,
        transform: RecordedTransformKind,
    },
    Hedge {
        primary_action_id: u64,
        transfer: RecordedTransfer,
    },
    Cancel {
        action_id: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedTransformKind {
    Remux,
    Segment,
    Transcode,
}

pub(super) fn capture(command: &PlannerCommand, privacy: &DecisionPrivacy) -> RecordedWarpCommand {
    match command {
        PlannerCommand::ProbeHead { .. }
        | PlannerCommand::Transfer(_)
        | PlannerCommand::Hedge { .. } => external(command, privacy),
        PlannerCommand::Promote { .. }
        | PlannerCommand::Transform { .. }
        | PlannerCommand::Cancel(_) => control(command, privacy),
    }
}

fn external(command: &PlannerCommand, privacy: &DecisionPrivacy) -> RecordedWarpCommand {
    match command {
        PlannerCommand::ProbeHead { post, source } => RecordedWarpCommand::ProbeHead {
            post_id: privacy.post(post.as_str()),
            source_id: privacy.source(source),
        },
        PlannerCommand::Transfer(value) => RecordedWarpCommand::Transfer {
            transfer: allocation::capture(value, privacy),
        },
        PlannerCommand::Hedge {
            primary,
            transfer: value,
        } => RecordedWarpCommand::Hedge {
            primary_action_id: primary.value(),
            transfer: allocation::capture(value, privacy),
        },
        _ => unreachable!("only source-bearing commands are routed here"),
    }
}

fn control(command: &PlannerCommand, privacy: &DecisionPrivacy) -> RecordedWarpCommand {
    match command {
        PlannerCommand::Promote { post, action } => RecordedWarpCommand::Promote {
            post_id: privacy.post(post.as_str()),
            action_id: action.value(),
        },
        PlannerCommand::Transform { post, kind } => RecordedWarpCommand::Transform {
            post_id: privacy.post(post.as_str()),
            transform: RecordedTransformKind::from(*kind),
        },
        PlannerCommand::Cancel(action) => RecordedWarpCommand::Cancel {
            action_id: action.value(),
        },
        _ => unreachable!("only local control commands are routed here"),
    }
}

impl RecordedWarpCommand {
    pub(super) fn projection(&self) -> (&str, &str, u64, u64) {
        match self {
            Self::ProbeHead { .. } | Self::Transfer { .. } | Self::Hedge { .. } => {
                self.external_projection()
            }
            Self::Promote { .. } | Self::Transform { .. } | Self::Cancel { .. } => {
                self.control_projection()
            }
        }
    }

    fn external_projection(&self) -> (&str, &str, u64, u64) {
        match self {
            Self::ProbeHead { source_id, .. } => ("head", source_id, 0, 0),
            Self::Transfer { transfer } => transfer.projection("transfer"),
            Self::Hedge { transfer, .. } => transfer.projection("hedge"),
            _ => unreachable!("only source-bearing commands are routed here"),
        }
    }

    fn control_projection(&self) -> (&str, &str, u64, u64) {
        match self {
            Self::Promote { .. } => ("promote", "", 0, 0),
            Self::Transform { .. } => ("transform", "", 0, 0),
            Self::Cancel { .. } => ("cancel", "", 0, 0),
            _ => unreachable!("only local control commands are routed here"),
        }
    }
}

impl From<TransformKind> for RecordedTransformKind {
    fn from(value: TransformKind) -> Self {
        match value {
            TransformKind::Remux => Self::Remux,
            TransformKind::Segment => Self::Segment,
            TransformKind::Transcode => Self::Transcode,
        }
    }
}
