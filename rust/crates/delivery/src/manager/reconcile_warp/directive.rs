//! Pure mapping from one planner result to executable WARP work.

use crate::manager::plan::{PlannedTransfer, PlannedTransferId, PlannedWork};
use ghostr_engine::adaptive::PlannerCommand;
use ghostr_engine::{ActionId, PostId};
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WarpDirective {
    None,
    ProbeHead {
        post: PostId,
        source: String,
    },
    Cancel(ActionId),
    Hedge {
        primary: ActionId,
        alternate: PlannedTransferId,
    },
    Unsupported {
        class: &'static str,
        cancel: Option<ActionId>,
    },
}

pub(crate) struct WarpExecution {
    pub(crate) transfers: Vec<PlannedTransfer>,
    pub(crate) retained: HashSet<ActionId>,
    pub(crate) retained_posts: HashSet<PostId>,
    pub(crate) emergency: bool,
    pub(crate) directive: WarpDirective,
}

pub(crate) fn execution(mut planned: PlannedWork) -> WarpExecution {
    let advanced = planned.warp.is_some();
    let command = planned
        .warp
        .as_ref()
        .and_then(|decision| decision.selected.as_ref())
        .map(|selected| &selected.command);
    let directive = directive_for(command, &planned.selected_transfers);
    let retained_posts = retained_posts(&planned, advanced);
    let transfers = match advanced {
        true => std::mem::take(&mut planned.selected_transfers),
        false => std::mem::take(&mut planned.transfers),
    };
    WarpExecution {
        transfers,
        retained: planned.retained,
        retained_posts,
        emergency: planned.emergency,
        directive,
    }
}

fn retained_posts(planned: &PlannedWork, advanced: bool) -> HashSet<PostId> {
    if !advanced {
        return planned
            .plan
            .retained
            .iter()
            .map(|item| item.post.clone())
            .collect();
    }
    planned
        .snapshot
        .iter()
        .flat_map(|snapshot| &snapshot.candidates)
        .filter(|candidate| {
            candidate
                .in_flight
                .iter()
                .any(|active| planned.retained.contains(&active.action_id))
        })
        .map(|candidate| candidate.post.clone())
        .collect()
}

pub(crate) fn directive_for(
    command: Option<&PlannerCommand>,
    selected: &[PlannedTransfer],
) -> WarpDirective {
    match command {
        None => WarpDirective::None,
        Some(PlannerCommand::ProbeHead { post, source }) => WarpDirective::ProbeHead {
            post: post.clone(),
            source: source.clone(),
        },
        Some(PlannerCommand::Cancel(action)) => WarpDirective::Cancel(*action),
        Some(command) => work_directive(command, selected),
    }
}

fn work_directive(command: &PlannerCommand, selected: &[PlannedTransfer]) -> WarpDirective {
    if let PlannerCommand::Hedge { primary, .. } = command {
        return hedge(*primary, selected);
    }
    if matches!(command, PlannerCommand::Transfer(_)) {
        return transfer(selected);
    }
    unsupported(command)
}

fn unsupported(command: &PlannerCommand) -> WarpDirective {
    match command {
        PlannerCommand::Promote { action, .. } => WarpDirective::Unsupported {
            class: "warp_live_promotion_backend_unavailable",
            cancel: Some(*action),
        },
        PlannerCommand::Transform { .. } => WarpDirective::Unsupported {
            class: "warp_transform_backend_unavailable",
            cancel: None,
        },
        PlannerCommand::ProbeHead { .. }
        | PlannerCommand::Cancel(_)
        | PlannerCommand::Hedge { .. }
        | PlannerCommand::Transfer(_) => {
            unreachable!("probe, cancel, and transfer commands are handled above")
        }
    }
}

fn hedge(primary: ActionId, selected: &[PlannedTransfer]) -> WarpDirective {
    selected.first().map_or(
        WarpDirective::Unsupported {
            class: "warp_hedge_transfer_unavailable",
            cancel: None,
        },
        |transfer| WarpDirective::Hedge {
            primary,
            alternate: transfer.id(),
        },
    )
}

fn transfer(selected: &[PlannedTransfer]) -> WarpDirective {
    match selected.is_empty() {
        false => WarpDirective::None,
        true => WarpDirective::Unsupported {
            class: "warp_transfer_unavailable",
            cancel: None,
        },
    }
}
