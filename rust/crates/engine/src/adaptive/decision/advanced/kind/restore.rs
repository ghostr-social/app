use super::{RecordedTransformKind, RecordedWarpActionKind};
use crate::adaptive::{ActionKind, TransformKind};
use crate::{ActionId, ByteRange};

impl RecordedWarpActionKind {
    pub(in crate::adaptive::decision) fn restore(&self) -> Option<ActionKind> {
        match self {
            Self::Head => Some(ActionKind::Head),
            Self::Prefix { .. } | Self::Tail { .. } | Self::FetchRange { .. } => {
                restore_range(self)
            }
            Self::FetchWhole { .. } | Self::CacheUpgrade { .. } => Some(restore_stored(self)),
            Self::Promote { .. }
            | Self::Transform { .. }
            | Self::Hedge { .. }
            | Self::Cancel { .. } => Some(restore_control(self)),
        }
    }
}

fn restore_range(value: &RecordedWarpActionKind) -> Option<ActionKind> {
    let (start, end) = match value {
        RecordedWarpActionKind::Prefix {
            bytes_start,
            bytes_end,
        }
        | RecordedWarpActionKind::Tail {
            bytes_start,
            bytes_end,
        }
        | RecordedWarpActionKind::FetchRange {
            bytes_start,
            bytes_end,
        } => (*bytes_start, *bytes_end),
        _ => unreachable!("only byte-range kinds are routed here"),
    };
    (start < end).then(|| restored_range_kind(value, ByteRange::new(start, end)))
}

fn restored_range_kind(value: &RecordedWarpActionKind, bytes: ByteRange) -> ActionKind {
    match value {
        RecordedWarpActionKind::Prefix { .. } => ActionKind::Prefix(bytes),
        RecordedWarpActionKind::Tail { .. } => ActionKind::Tail(bytes),
        RecordedWarpActionKind::FetchRange { .. } => ActionKind::FetchRange(bytes),
        _ => unreachable!("only byte-range kinds are routed here"),
    }
}

fn restore_stored(value: &RecordedWarpActionKind) -> ActionKind {
    match value {
        RecordedWarpActionKind::FetchWhole { maximum_bytes } => ActionKind::FetchWhole {
            maximum_bytes: *maximum_bytes,
        },
        RecordedWarpActionKind::CacheUpgrade {
            bytes_start,
            bytes_end,
        } => ActionKind::CacheUpgrade(ByteRange::new(*bytes_start, *bytes_end)),
        _ => unreachable!("only storage-bound kinds are routed here"),
    }
}

fn restore_control(value: &RecordedWarpActionKind) -> ActionKind {
    match value {
        RecordedWarpActionKind::Promote { .. } | RecordedWarpActionKind::Transform { .. } => {
            restore_mutation(value)
        }
        RecordedWarpActionKind::Hedge { .. } | RecordedWarpActionKind::Cancel { .. } => {
            restore_request_control(value)
        }
        _ => unreachable!("only control kinds are routed here"),
    }
}

fn restore_mutation(value: &RecordedWarpActionKind) -> ActionKind {
    match value {
        RecordedWarpActionKind::Promote {
            active_action_id,
            maximum_bytes,
        } => ActionKind::Promote {
            active: ActionId::new(*active_action_id),
            maximum_bytes: *maximum_bytes,
        },
        RecordedWarpActionKind::Transform { transform } => {
            ActionKind::Transform(restore_transform(*transform))
        }
        _ => unreachable!("only mutation controls are routed here"),
    }
}

fn restore_request_control(value: &RecordedWarpActionKind) -> ActionKind {
    match value {
        RecordedWarpActionKind::Hedge {
            primary_action_id,
            alternate_source_id,
        } => ActionKind::Hedge {
            primary: ActionId::new(*primary_action_id),
            alternate: alternate_source_id.clone(),
        },
        RecordedWarpActionKind::Cancel { action_id } => {
            ActionKind::Cancel(ActionId::new(*action_id))
        }
        _ => unreachable!("only request controls are routed here"),
    }
}

fn restore_transform(value: RecordedTransformKind) -> TransformKind {
    match value {
        RecordedTransformKind::Remux => TransformKind::Remux,
        RecordedTransformKind::Segment => TransformKind::Segment,
        RecordedTransformKind::Transcode => TransformKind::Transcode,
    }
}
