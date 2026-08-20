//! Converts one planner result into the exact work the manager may execute.

use crate::manager::plan::{PlannedTransfer, PlannedTransferId, PlannedWork};
use crate::manager::transfers::spawn_probe;
use crate::manager::{time, DeliveryWorker};
use ghostr_engine::adaptive::{DecisionOutcome, PlannerCommand};
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
        Some(PlannerCommand::Hedge { primary, .. }) => hedge(*primary, selected),
        Some(PlannerCommand::Transfer(_)) => transfer(selected),
        Some(PlannerCommand::Promote { action, .. }) => WarpDirective::Unsupported {
            class: "warp_live_promotion_backend_unavailable",
            cancel: Some(*action),
        },
        Some(PlannerCommand::Transform { .. }) => WarpDirective::Unsupported {
            class: "warp_transform_backend_unavailable",
            cancel: None,
        },
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

impl DeliveryWorker {
    pub(super) fn apply_warp_directive(&mut self, directive: &WarpDirective) {
        match directive {
            WarpDirective::ProbeHead { post, source } => self.launch_selected_probe(post, source),
            WarpDirective::Cancel(action) => self.cancel_selected(*action),
            WarpDirective::Unsupported { class, cancel } => {
                cancel.iter().for_each(|action| {
                    self.downloads.cancel_action(*action);
                });
                self.fail_selected(class);
            }
            WarpDirective::None | WarpDirective::Hedge { .. } => {}
        }
    }

    fn launch_selected_probe(&mut self, post: &PostId, source: &str) {
        if self
            .probes
            .claim_selected(self.state.catalog(), &self.retry, post, source)
        {
            spawn_probe(self.ctx.clone(), post.clone(), source.to_owned());
        }
    }

    fn cancel_selected(&mut self, action: ActionId) {
        let outcome = match self.downloads.cancel_action(action) {
            true => DecisionOutcome::Succeeded {
                bytes: 0,
                elapsed_ms: 0,
            },
            false => DecisionOutcome::Failed {
                class: "warp_cancel_action_missing".into(),
                elapsed_ms: 0,
            },
        };
        self.commands.resolve_latest_decision(outcome);
    }

    fn fail_selected(&self, class: &str) {
        self.commands
            .resolve_latest_decision(DecisionOutcome::Failed {
                class: class.to_owned(),
                elapsed_ms: 0,
            });
    }

    pub(super) fn link_selected_hedge(
        &mut self,
        directive: &WarpDirective,
        alternate: &PlannedTransferId,
        action: ActionId,
    ) {
        let WarpDirective::Hedge {
            primary,
            alternate: selected,
        } = directive
        else {
            return;
        };
        if selected != alternate || self.downloads.link_hedge(*primary, action) {
            return;
        }
        self.downloads.cancel_action(action);
        self.commands.resolve_decision(
            action,
            DecisionOutcome::Failed {
                class: "warp_hedge_primary_missing".into(),
                elapsed_ms: 0,
            },
            time::unix_time_ms(),
        );
    }
}
