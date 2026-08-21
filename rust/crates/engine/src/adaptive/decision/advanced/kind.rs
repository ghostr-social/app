use super::super::privacy::DecisionPrivacy;
use super::command::RecordedTransformKind;
use crate::adaptive::ActionKind;
use serde::{Deserialize, Serialize};

mod restore;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecordedWarpActionKind {
    Head,
    Prefix {
        bytes_start: u64,
        bytes_end: u64,
    },
    Tail {
        bytes_start: u64,
        bytes_end: u64,
    },
    FetchRange {
        bytes_start: u64,
        bytes_end: u64,
    },
    FetchWhole {
        maximum_bytes: u64,
    },
    Promote {
        active_action_id: u64,
        maximum_bytes: u64,
    },
    Transform {
        transform: RecordedTransformKind,
    },
    CacheUpgrade {
        bytes_start: u64,
        bytes_end: u64,
    },
    Hedge {
        primary_action_id: u64,
        alternate_source_id: String,
    },
    Cancel {
        action_id: u64,
    },
}

pub(super) fn capture(kind: &ActionKind, privacy: &DecisionPrivacy) -> RecordedWarpActionKind {
    match kind {
        ActionKind::Head => RecordedWarpActionKind::Head,
        ActionKind::Prefix(_) | ActionKind::Tail(_) | ActionKind::FetchRange(_) => ranged(kind),
        ActionKind::FetchWhole { .. } | ActionKind::CacheUpgrade(_) => stored(kind),
        ActionKind::Promote { .. }
        | ActionKind::Transform(_)
        | ActionKind::Hedge { .. }
        | ActionKind::Cancel(_) => controlled(kind, privacy),
    }
}

fn ranged(kind: &ActionKind) -> RecordedWarpActionKind {
    match kind {
        ActionKind::Prefix(bytes) => RecordedWarpActionKind::Prefix {
            bytes_start: bytes.start,
            bytes_end: bytes.end,
        },
        ActionKind::Tail(bytes) => RecordedWarpActionKind::Tail {
            bytes_start: bytes.start,
            bytes_end: bytes.end,
        },
        ActionKind::FetchRange(bytes) => RecordedWarpActionKind::FetchRange {
            bytes_start: bytes.start,
            bytes_end: bytes.end,
        },
        _ => unreachable!("only byte-range kinds are routed here"),
    }
}

fn stored(kind: &ActionKind) -> RecordedWarpActionKind {
    match kind {
        ActionKind::FetchWhole { maximum_bytes } => RecordedWarpActionKind::FetchWhole {
            maximum_bytes: *maximum_bytes,
        },
        ActionKind::CacheUpgrade(bytes) => RecordedWarpActionKind::CacheUpgrade {
            bytes_start: bytes.start,
            bytes_end: bytes.end,
        },
        _ => unreachable!("only storage-bound kinds are routed here"),
    }
}

fn controlled(kind: &ActionKind, privacy: &DecisionPrivacy) -> RecordedWarpActionKind {
    match kind {
        ActionKind::Promote {
            active,
            maximum_bytes,
        } => RecordedWarpActionKind::Promote {
            active_action_id: active.value(),
            maximum_bytes: *maximum_bytes,
        },
        ActionKind::Transform(kind) => RecordedWarpActionKind::Transform {
            transform: RecordedTransformKind::from(*kind),
        },
        ActionKind::Hedge { primary, alternate } => RecordedWarpActionKind::Hedge {
            primary_action_id: primary.value(),
            alternate_source_id: privacy.source(alternate),
        },
        ActionKind::Cancel(action) => RecordedWarpActionKind::Cancel {
            action_id: action.value(),
        },
        _ => unreachable!("only control kinds are routed here"),
    }
}
