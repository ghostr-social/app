use super::super::privacy::DecisionPrivacy;
use super::RecordedHlsBootstrapStage;
use crate::adaptive::{PlannerCommand, TransformKind};
use serde::{Deserialize, Serialize};

mod allocation;
pub(super) mod request;
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
        authority: RecordedPreemptionAuthority,
    },
    Transfer {
        transfer: RecordedTransfer,
    },
    FetchHlsBootstrap {
        post_id: String,
        stage: RecordedHlsBootstrapStage,
        source_id: String,
        maximum_bytes: u64,
        committed_until_ms: u64,
    },
    Promote {
        post_id: String,
        action_id: u64,
        source_id: String,
        grant: RecordedPromotionGrant,
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
        | PlannerCommand::FetchHlsBootstrap { .. }
        | PlannerCommand::Hedge { .. } => external(command, privacy),
        PlannerCommand::Promote { .. }
        | PlannerCommand::Transform { .. }
        | PlannerCommand::Cancel(_) => control(command, privacy),
    }
}

fn external(command: &PlannerCommand, privacy: &DecisionPrivacy) -> RecordedWarpCommand {
    match command {
        PlannerCommand::ProbeHead {
            post,
            source,
            authority,
        } => RecordedWarpCommand::ProbeHead {
            post_id: privacy.post(post.as_str()),
            source_id: privacy.source(source),
            authority: (*authority).into(),
        },
        PlannerCommand::Transfer(value) => RecordedWarpCommand::Transfer {
            transfer: allocation::capture(value, privacy),
        },
        PlannerCommand::FetchHlsBootstrap {
            post,
            stage,
            source,
            maximum_bytes,
            committed_until_ms,
        } => RecordedWarpCommand::FetchHlsBootstrap {
            post_id: privacy.post(post.as_str()),
            stage: (*stage).into(),
            source_id: privacy.source(source),
            maximum_bytes: *maximum_bytes,
            committed_until_ms: *committed_until_ms,
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
        PlannerCommand::Promote {
            post,
            action,
            source,
            grant,
        } => RecordedWarpCommand::Promote {
            post_id: privacy.post(post.as_str()),
            action_id: action.value(),
            source_id: privacy.source(source),
            grant: (*grant).into(),
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
    pub(in crate::adaptive::decision) fn projection(&self) -> (&str, &str, u64, u64) {
        match self {
            Self::ProbeHead { .. }
            | Self::Transfer { .. }
            | Self::FetchHlsBootstrap { .. }
            | Self::Hedge { .. } => self.external_projection(),
            Self::Promote { .. } | Self::Transform { .. } | Self::Cancel { .. } => {
                self.control_projection()
            }
        }
    }

    fn external_projection(&self) -> (&str, &str, u64, u64) {
        match self {
            Self::ProbeHead { source_id, .. } => ("head", source_id, 0, 0),
            Self::Transfer { transfer } => transfer.projection("transfer"),
            Self::FetchHlsBootstrap {
                source_id,
                maximum_bytes,
                ..
            } => ("hls_bootstrap", source_id, 0, *maximum_bytes),
            Self::Hedge { transfer, .. } => transfer.projection("hedge"),
            _ => unreachable!("only source-bearing commands are routed here"),
        }
    }

    fn control_projection(&self) -> (&str, &str, u64, u64) {
        match self {
            Self::Promote {
                source_id, grant, ..
            } => ("promote", source_id, 0, grant.maximum_bytes),
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
