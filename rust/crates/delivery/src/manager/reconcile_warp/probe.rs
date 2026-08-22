use super::DeliveryWorker;
use crate::delivery_events::{DecisionClaim, DecisionToken};
use crate::manager::selected_commit::{CommitResult, SelectedCommit};
use crate::manager::time;
use crate::manager::transfers::{spawn_probe, ProbeLaunch};
use crate::probe::pool::ProbeClaimQuery;
use ghostr_engine::adaptive::{DecisionOutcome, PreemptionAuthority, ResourceCost};
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;

struct ClaimedProbe {
    identity: TransferIdentity,
    authority: PreemptionAuthority,
}

pub(super) struct SelectedProbe<'a> {
    pub(super) post: &'a PostId,
    pub(super) source: &'a str,
    pub(super) authority: PreemptionAuthority,
}

struct ProbeCommit {
    probe: ClaimedProbe,
    decision: DecisionClaim,
    observed_at_ms: u64,
}

impl DeliveryWorker {
    pub(super) fn launch_selected_probe(
        &mut self,
        selected: SelectedProbe<'_>,
        decision: Option<DecisionToken>,
        commit: &mut Option<SelectedCommit>,
    ) {
        let Some(token) = decision else {
            return;
        };
        let query = ProbeClaimQuery::new(
            self.state.catalog(),
            &self.retry,
            selected.post,
            selected.source,
        );
        match self.probes.claim_selected(query) {
            Ok(identity) => self.claim_probe(
                token,
                ClaimedProbe {
                    identity,
                    authority: selected.authority,
                },
                commit,
            ),
            Err(reason) => {
                self.commands
                    .resolve_decision_token(&token, DecisionOutcome::ClaimRefused { reason });
            }
        }
    }

    fn claim_probe(
        &mut self,
        token: DecisionToken,
        probe: ClaimedProbe,
        commit: &mut Option<SelectedCommit>,
    ) {
        let observed_at_ms = time::unix_time_ms();
        match self
            .commands
            .claim_decision(token, &probe.identity, observed_at_ms)
        {
            Ok(decision) => self.commit_probe(
                ProbeCommit {
                    probe,
                    decision,
                    observed_at_ms,
                },
                commit,
            ),
            Err(token) => {
                self.probes.release(probe.identity.post());
                self.fail_selected("warp_head_probe_claim_rejected", Some(token));
            }
        }
    }

    fn commit_probe(&mut self, owned: ProbeCommit, commit: &mut Option<SelectedCommit>) {
        let resources = ResourceCost::new(0, 0, 0, 1);
        if self.commit_selected(commit, resources, owned.observed_at_ms) == CommitResult::Rejected {
            self.reject_probe_commit(owned);
            return;
        }
        spawn_probe(
            self.ctx.clone(),
            ProbeLaunch {
                post: owned.probe.identity.post().clone(),
                url: owned.probe.identity.source().as_str().to_owned(),
                authority: owned.probe.authority,
                decision: owned.decision,
            },
        );
        self.request_immediate_replan();
    }

    fn reject_probe_commit(&mut self, owned: ProbeCommit) {
        self.probes.release(owned.probe.identity.post());
        self.commands.resolve_decision_claim(
            owned.decision,
            DecisionOutcome::Failed {
                class: "warp_resource_commit_rejected".into(),
                elapsed_ms: 0,
            },
            owned.observed_at_ms,
        );
    }
}
