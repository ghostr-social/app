//! Converts one planner result into the exact work the manager may execute.

mod directive;

#[cfg(test)]
pub(crate) use directive::directive_for;
pub(crate) use directive::{execution, WarpDirective};

use crate::manager::plan::PlannedTransferId;
use crate::manager::transfers::spawn_probe;
use crate::manager::{time, DeliveryWorker};
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::{ActionId, PostId};

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
