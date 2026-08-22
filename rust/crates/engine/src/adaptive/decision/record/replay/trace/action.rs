use crate::adaptive::{
    RecordedRetrievalRequest, RecordedWarpAction, RecordedWarpActionKind, RecordedWarpCommand,
    RecordedWholeBodyContract,
};

pub(super) fn coherent(action: &RecordedWarpAction) -> bool {
    post_matches(action) && kind_matches(action)
}

fn post_matches(action: &RecordedWarpAction) -> bool {
    let post = match &action.command {
        RecordedWarpCommand::ProbeHead { post_id, .. }
        | RecordedWarpCommand::FetchHlsBootstrap { post_id, .. }
        | RecordedWarpCommand::Promote { post_id, .. }
        | RecordedWarpCommand::Transform { post_id, .. } => Some(post_id),
        RecordedWarpCommand::Transfer { transfer }
        | RecordedWarpCommand::Hedge { transfer, .. } => Some(&transfer.post_id),
        RecordedWarpCommand::Cancel { .. } => None,
    };
    post.is_none_or(|post| post == &action.post_id)
}

fn kind_matches(action: &RecordedWarpAction) -> bool {
    match &action.kind {
        RecordedWarpActionKind::Head
        | RecordedWarpActionKind::Prefix { .. }
        | RecordedWarpActionKind::Tail { .. }
        | RecordedWarpActionKind::FetchRange { .. } => retrieval_matches(action),
        RecordedWarpActionKind::FetchWhole { .. }
        | RecordedWarpActionKind::HlsBootstrap { .. }
        | RecordedWarpActionKind::CacheUpgrade { .. } => stored_matches(action),
        _ => control_matches(action),
    }
}

fn retrieval_matches(action: &RecordedWarpAction) -> bool {
    match (&action.kind, &action.command) {
        (RecordedWarpActionKind::Head, RecordedWarpCommand::ProbeHead { .. }) => true,
        (
            RecordedWarpActionKind::Prefix {
                bytes_start,
                bytes_end,
            },
            command,
        )
        | (
            RecordedWarpActionKind::Tail {
                bytes_start,
                bytes_end,
            },
            command,
        )
        | (
            RecordedWarpActionKind::FetchRange {
                bytes_start,
                bytes_end,
            },
            command,
        ) => range_matches(command, *bytes_start, *bytes_end),
        _ => false,
    }
}

fn stored_matches(action: &RecordedWarpAction) -> bool {
    match (&action.kind, &action.command) {
        (RecordedWarpActionKind::FetchWhole { maximum_bytes }, command) => {
            whole_matches(command, *maximum_bytes)
        }
        (
            RecordedWarpActionKind::HlsBootstrap {
                stage,
                cursor,
                maximum_bytes,
            },
            RecordedWarpCommand::FetchHlsBootstrap {
                stage: command_stage,
                cursor: command_cursor,
                maximum_bytes: command_maximum,
                ..
            },
        ) => stage == command_stage && cursor == command_cursor && maximum_bytes == command_maximum,
        (
            RecordedWarpActionKind::CacheUpgrade {
                bytes_start,
                bytes_end,
            },
            command,
        ) => range_matches(command, *bytes_start, *bytes_end),
        _ => false,
    }
}

fn control_matches(action: &RecordedWarpAction) -> bool {
    match (&action.kind, &action.command) {
        (
            RecordedWarpActionKind::Promote {
                active_action_id,
                maximum_bytes,
            },
            command,
        ) => promote_matches(command, *active_action_id, *maximum_bytes),
        (RecordedWarpActionKind::Transform { transform }, command) => {
            matches!(command, RecordedWarpCommand::Transform { transform: other, .. } if other == transform)
        }
        (
            RecordedWarpActionKind::Hedge {
                primary_action_id,
                alternate_source_id,
            },
            command,
        ) => hedge_matches(command, *primary_action_id, alternate_source_id),
        (
            RecordedWarpActionKind::Cancel { action_id },
            RecordedWarpCommand::Cancel { action_id: other },
        ) => action_id == other,
        _ => false,
    }
}

fn range_matches(command: &RecordedWarpCommand, start: u64, end: u64) -> bool {
    let RecordedWarpCommand::Transfer { transfer } = command else {
        return false;
    };
    matches!(
        transfer.request,
        RecordedRetrievalRequest::FetchRange { bytes_start, bytes_end, .. }
            if bytes_start == start && bytes_end == end
    )
}

fn whole_matches(command: &RecordedWarpCommand, maximum: u64) -> bool {
    let RecordedWarpCommand::Transfer { transfer } = command else {
        return false;
    };
    whole_bytes(transfer.request) == Some(maximum)
}

fn whole_bytes(request: RecordedRetrievalRequest) -> Option<u64> {
    let RecordedRetrievalRequest::FetchWhole { contract, .. } = request else {
        return None;
    };
    Some(match contract {
        RecordedWholeBodyContract::Exact { expected_bytes } => expected_bytes,
        RecordedWholeBodyContract::Capped { maximum_bytes } => maximum_bytes,
    })
}

fn promote_matches(command: &RecordedWarpCommand, active: u64, maximum: u64) -> bool {
    matches!(
        command,
        RecordedWarpCommand::Promote { action_id, grant, .. }
            if *action_id == active && grant.maximum_bytes == maximum
    )
}

fn hedge_matches(command: &RecordedWarpCommand, primary: u64, alternate: &str) -> bool {
    matches!(
        command,
        RecordedWarpCommand::Hedge { primary_action_id, transfer }
            if *primary_action_id == primary && transfer.source_id == alternate
    )
}
