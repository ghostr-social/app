//! Converts one planner result into the exact work the manager may execute.

mod directive;

#[cfg(test)]
pub(crate) use directive::directive_for;
pub(crate) use directive::{execution, WarpDirective};

use crate::delivery_events::DecisionToken;
use crate::manager::plan::PlannedTransferId;
use crate::manager::transfers::{spawn_probe, ProbeLaunch};
use crate::manager::{time, DeliveryWorker};
use crate::probe::pool::ProbeClaimQuery;
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::{ActionId, PostId};

impl DeliveryWorker {
    pub(super) fn apply_warp_directive(
        &mut self,
        directive: &WarpDirective,
        decision: &mut Option<DecisionToken>,
    ) {
        match directive {
            WarpDirective::ProbeHead { post, source } => {
                self.launch_selected_probe(post, source, decision.take());
            }
            WarpDirective::Cancel(action) => self.cancel_selected(*action, decision.take()),
            WarpDirective::Unsupported { class, cancel } => {
                cancel.iter().for_each(|action| {
                    self.downloads.cancel_action(*action);
                });
                self.fail_selected(class, decision.take());
            }
            WarpDirective::None | WarpDirective::Hedge { .. } => {}
        }
    }

    fn launch_selected_probe(
        &mut self,
        post: &PostId,
        source: &str,
        decision: Option<DecisionToken>,
    ) {
        let Some(token) = decision else {
            return;
        };
        let query = ProbeClaimQuery::new(self.state.catalog(), &self.retry, post, source);
        match self.probes.claim_selected(query) {
            Ok(identity) => self.claim_and_spawn_probe(token, identity),
            Err(reason) => {
                self.commands
                    .resolve_decision_token(&token, DecisionOutcome::ClaimRefused { reason });
            }
        }
    }

    fn claim_and_spawn_probe(
        &mut self,
        token: DecisionToken,
        identity: ghostr_engine::representation::TransferIdentity,
    ) {
        let started_at_ms = time::unix_time_ms();
        match self
            .commands
            .claim_decision(token, &identity, started_at_ms)
        {
            Ok(decision) => spawn_probe(
                self.ctx.clone(),
                ProbeLaunch {
                    post: identity.post().clone(),
                    url: identity.source().as_str().to_owned(),
                    decision,
                },
            ),
            Err(token) => {
                self.probes.release(identity.post());
                self.fail_selected("warp_head_probe_claim_rejected", Some(token));
            }
        }
    }

    fn cancel_selected(&mut self, action: ActionId, decision: Option<DecisionToken>) {
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
        if let Some(token) = decision {
            self.commands.resolve_decision_token(&token, outcome);
        }
    }

    fn fail_selected(&self, class: &str, decision: Option<DecisionToken>) {
        if let Some(token) = decision {
            self.commands.resolve_decision_token(
                &token,
                DecisionOutcome::Failed {
                    class: class.to_owned(),
                    elapsed_ms: 0,
                },
            );
        }
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
