//! Pure mapping from one planner result to executable WARP work.

use crate::manager::plan::{PlannedTransfer, PlannedTransferId, PlannedWork};
use ghostr_engine::adaptive::{PlannerCommand, PromotionGrant, TransformKind};
use ghostr_engine::{ActionId, PostId};
use std::collections::HashSet;

mod work;
use work::work_directive;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WarpDirective {
    None,
    ProbeHead {
        post: PostId,
        source: String,
        authority: ghostr_engine::adaptive::PreemptionAuthority,
    },
    Cancel(ActionId),
    Hedge {
        primary: ActionId,
        alternate: PlannedTransferId,
    },
    Promote {
        post: PostId,
        action: ActionId,
        source: String,
        grant: PromotionGrant,
    },
    Transform {
        post: PostId,
        kind: TransformKind,
    },
    HlsBootstrap {
        post: PostId,
        stage: ghostr_engine::adaptive::HlsBootstrapStage,
        source: String,
        cursor: ghostr_engine::adaptive::HlsObjectCursor,
        maximum_bytes: u64,
        committed_until_ms: u64,
    },
    Unsupported {
        class: &'static str,
    },
}

pub(crate) struct WarpExecution {
    pub(crate) transfers: Vec<PlannedTransfer>,
    pub(crate) retained: HashSet<ActionId>,
    pub(crate) retained_posts: HashSet<PostId>,
    pub(crate) emergency: bool,
    pub(crate) directive: WarpDirective,
    pub(crate) selected: Option<ghostr_engine::adaptive::GeneratedAction>,
}

pub(crate) fn execution(mut planned: PlannedWork) -> WarpExecution {
    let advanced = planned.warp.is_some();
    let selected = planned
        .warp
        .as_ref()
        .and_then(|decision| decision.selected.as_ref())
        .cloned();
    let command = selected.as_ref().map(|action| &action.command);
    let directive = directive_for(command, &planned.selected_transfers);
    let retained_posts = retained_posts(&planned, advanced);
    let transfers = if advanced {
        core::mem::take(&mut planned.selected_transfers)
    } else {
        core::mem::take(&mut planned.transfers)
    };
    WarpExecution {
        transfers,
        retained: planned.retained,
        retained_posts,
        emergency: planned.emergency,
        directive,
        selected,
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
        Some(PlannerCommand::ProbeHead {
            post,
            source,
            authority,
        }) => WarpDirective::ProbeHead {
            post: post.clone(),
            source: source.clone(),
            authority: *authority,
        },
        Some(PlannerCommand::Cancel(action)) => WarpDirective::Cancel(*action),
        Some(command) => work_directive(command, selected),
    }
}
