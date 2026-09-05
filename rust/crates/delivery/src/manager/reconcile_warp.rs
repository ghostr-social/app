//! Converts one planner result into the exact work the manager may execute.

mod directive;
mod hls;
mod probe;
mod promotion;
mod transform;

pub(crate) use directive::{execution, WarpDirective};
use probe::SelectedProbe;

use crate::delivery_events::DecisionToken;
use crate::manager::plan::PlannedTransferId;
use crate::manager::selected_commit::{CommitResult, SelectedCommit};
use crate::manager::{time, DeliveryWorker};
use ghostr_engine::adaptive::{DecisionOutcome, ResourceCost};
use ghostr_engine::origin_model::RequestMethod;
use ghostr_engine::ActionId;

#[derive(Clone, Copy)]
enum CancelCommit {
    Cancelled,
    Missing,
    ResourceRejected,
}

impl DeliveryWorker {
    pub(super) async fn apply_warp_directive(
        &mut self,
        directive: &WarpDirective,
        decision: &mut Option<DecisionToken>,
        commit: &mut Option<SelectedCommit>,
        observed_at_ms: u64,
    ) {
        match directive {
            WarpDirective::ProbeHead {
                post,
                source,
                authority,
            } => {
                let profile = commit
                    .as_ref()
                    .and_then(SelectedCommit::attempt_profile)
                    .filter(|profile| profile.request().method() == RequestMethod::Head);
                let Some(profile) = profile else {
                    self.fail_selected("warp_head_profile_missing", decision.take());
                    return;
                };
                let selected = SelectedProbe {
                    post,
                    source,
                    authority: *authority,
                    observed_at_ms,
                    profile,
                };
                self.launch_selected_probe(selected, decision.take(), commit);
            }
            WarpDirective::Cancel(action) => self.cancel_selected(*action, decision.take(), commit),
            WarpDirective::Promote { .. } => {
                self.promote_selected(directive, decision.take(), commit)
                    .await;
            }
            WarpDirective::Transform { .. } => {
                self.transform_selected(directive, decision.take(), commit)
                    .await;
            }
            WarpDirective::HlsBootstrap { .. } => {
                self.launch_selected_hls(directive, decision.take(), commit);
            }
            WarpDirective::Unsupported { class } => self.fail_selected(class, decision.take()),
            WarpDirective::None | WarpDirective::Hedge { .. } => {}
        }
    }

    fn cancel_selected(
        &mut self,
        action: ActionId,
        decision: Option<DecisionToken>,
        commit: &mut Option<SelectedCommit>,
    ) {
        let outcome = cancel_outcome(self.commit_cancel(action, commit));
        if let Some(token) = decision {
            self.commands.resolve_decision_token(&token, outcome);
        }
    }

    fn commit_cancel(
        &mut self,
        action: ActionId,
        commit: &mut Option<SelectedCommit>,
    ) -> CancelCommit {
        if !self.downloads.can_cancel_action(action) {
            return CancelCommit::Missing;
        }
        let resources = ResourceCost::new(0, 0, 0, 0);
        if self.commit_selected(commit, resources, time::unix_time_ms()) == CommitResult::Rejected {
            return CancelCommit::ResourceRejected;
        }
        let cancelled = self.downloads.cancel_action(action);
        if cancelled {
            self.request_immediate_replan();
            return CancelCommit::Cancelled;
        }
        CancelCommit::Missing
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
        self.downloads.release_hedge_authorization(*primary);
        self.downloads.cancel_hedge_loser(action);
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

fn cancel_outcome(commit: CancelCommit) -> DecisionOutcome {
    match commit {
        CancelCommit::Cancelled => DecisionOutcome::Succeeded {
            bytes: 0,
            elapsed_ms: 0,
        },
        CancelCommit::Missing => DecisionOutcome::Superseded,
        CancelCommit::ResourceRejected => DecisionOutcome::Failed {
            class: "warp_resource_commit_rejected".into(),
            elapsed_ms: 0,
        },
    }
}

#[cfg(test)]
#[path = "reconcile_warp_axiom_test.rs"]
pub(crate) mod axiom_test_support;
