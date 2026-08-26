use super::WarpDirective;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::adaptive::PlannerCommand;
use ghostr_engine::ActionId;

pub(super) fn work_directive(
    command: &PlannerCommand,
    selected: &[PlannedTransfer],
) -> WarpDirective {
    match command {
        PlannerCommand::Hedge { primary, .. } => hedge(*primary, selected),
        PlannerCommand::Transfer(_) => transfer(selected),
        PlannerCommand::Promote { .. } => promote(command),
        PlannerCommand::Transform { post, kind } => WarpDirective::Transform {
            post: post.clone(),
            kind: *kind,
        },
        PlannerCommand::FetchHlsBootstrap { .. } => hls(command),
        PlannerCommand::ProbeHead { .. } | PlannerCommand::Cancel(_) => unsupported(),
    }
}

fn promote(command: &PlannerCommand) -> WarpDirective {
    let PlannerCommand::Promote {
        post,
        action,
        source,
        grant,
    } = command
    else {
        unreachable!("only promotion commands are routed here")
    };
    WarpDirective::Promote {
        post: post.clone(),
        action: *action,
        source: source.clone(),
        grant: *grant,
    }
}

fn hls(command: &PlannerCommand) -> WarpDirective {
    let PlannerCommand::FetchHlsBootstrap {
        post,
        stage,
        source,
        cursor,
        maximum_bytes,
        committed_until_ms,
    } = command
    else {
        unreachable!("only HLS bootstrap commands are routed here")
    };
    WarpDirective::HlsBootstrap {
        post: post.clone(),
        stage: *stage,
        source: source.clone(),
        cursor: *cursor,
        maximum_bytes: *maximum_bytes,
        committed_until_ms: *committed_until_ms,
    }
}

fn hedge(primary: ActionId, selected: &[PlannedTransfer]) -> WarpDirective {
    selected.first().map_or(
        WarpDirective::Unsupported {
            class: "warp_hedge_transfer_unavailable",
        },
        |transfer| WarpDirective::Hedge {
            primary,
            alternate: transfer.id(),
        },
    )
}

fn transfer(selected: &[PlannedTransfer]) -> WarpDirective {
    if selected.is_empty() {
        WarpDirective::Unsupported {
            class: "warp_transfer_unavailable",
        }
    } else {
        WarpDirective::None
    }
}

fn unsupported() -> WarpDirective {
    unreachable!("probe and cancel commands are handled before work mapping")
}
