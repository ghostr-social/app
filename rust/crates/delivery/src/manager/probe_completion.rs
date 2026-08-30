//! Completion policy for media-size probes.

use crate::delivery_events::DecisionClaim;
use crate::manager::failure::{origin_failure_class, FailureClass};
use crate::manager::transfers::{ProbeDone, ProbeObservation};
use crate::manager::DeliveryWorker;
use crate::probe::media::{is_usefulness_timeout, ProbeResult};
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::representation::TransferIdentity;
use ghostr_net::media_log_identity::MediaLogIdentity;
use log::warn;

mod generation;

impl DeliveryWorker {
    pub(super) async fn finish_probe(&mut self, done: ProbeDone) {
        let ProbeDone {
            observation,
            decision,
        } = done;
        let observed_at_ms = probe_observed_at(&observation);
        let identity =
            self.probes
                .current_identity(self.state.catalog(), &observation.post, &observation.url);
        let Some(identity) = identity else {
            self.probes.release(&observation.post);
            self.resolve_probe_claim(decision, DecisionOutcome::Superseded, observed_at_ms);
            return;
        };
        self.keeper.note_probe(&observation);
        let outcome = match observation.outcome {
            Ok(result) => self.finish_probe_result(&identity, result).await,
            Err(error) => self.finish_probe_error(&identity, &error),
        };
        self.resolve_probe_claim(decision, outcome, observed_at_ms);
    }

    pub(in crate::manager) async fn finish_probe_result(
        &mut self,
        identity: &TransferIdentity,
        result: ProbeResult,
    ) -> DecisionOutcome {
        let stamp = match self.absorb_probe(identity, &result).await {
            Ok(Some(stamp)) => stamp,
            Ok(None) => {
                self.probes.release(identity.post());
                return DecisionOutcome::Superseded;
            }
            Err(error) => return self.failed_probe_store(identity, &error),
        };
        let observed_size = result.content_length.is_some_and(|length| length > 0);
        self.probes.learned_probe(identity, stamp, observed_size);
        if result.content_length.is_some_and(|length| length > 0) {
            return Self::finish_sized_probe(&result);
        }
        self.finish_missing_length(identity)
    }

    fn failed_probe_store(
        &mut self,
        identity: &TransferIdentity,
        error: &anyhow::Error,
    ) -> DecisionOutcome {
        self.probes.release(identity.post());
        log::warn!("Could not apply HEAD generation: {error:#}");
        DecisionOutcome::Failed {
            class: "warp_head_generation_store_failure".into(),
            elapsed_ms: 0,
        }
    }

    fn finish_sized_probe(result: &ProbeResult) -> DecisionOutcome {
        DecisionOutcome::HeadObserved {
            content_length: result.content_length.unwrap_or_default(),
            accept_ranges: result.accept_ranges,
            elapsed_ms: 0,
        }
    }

    fn finish_missing_length(&mut self, identity: &TransferIdentity) -> DecisionOutcome {
        let deferred = self.defer_probe_to_body(identity);
        if !deferred {
            self.probes.release(identity.post());
            let source = identity.source().as_str();
            warn!(
                "Probe did not reveal a usable content length for {}",
                MediaLogIdentity::from_url(source)
            );
            self.note_failed_attempt(identity.post(), source, FailureClass::Transient);
        }
        DecisionOutcome::Failed {
            class: if deferred {
                "warp_head_probe_deferred_to_body"
            } else {
                "warp_head_probe_missing_length"
            }
            .into(),
            elapsed_ms: 0,
        }
    }

    fn finish_probe_error(
        &mut self,
        identity: &TransferIdentity,
        error: &anyhow::Error,
    ) -> DecisionOutcome {
        if is_usefulness_timeout(error) {
            return self.finish_expired_probe(identity);
        }
        let Some(class) = origin_failure_class(error) else {
            self.probes.release(identity.post());
            return DecisionOutcome::Failed {
                class: "warp_head_probe_admission_exhausted".into(),
                elapsed_ms: 0,
            };
        };
        warn!(
            "Probe failed for {} ({class:?})",
            MediaLogIdentity::from_url(identity.source().as_str())
        );
        if !self.defer_probe_to_body(identity) {
            self.probes.release(identity.post());
            self.note_failed_attempt(identity.post(), identity.source().as_str(), class);
        }
        DecisionOutcome::Failed {
            class: failure_name(class).into(),
            elapsed_ms: 0,
        }
    }

    fn finish_expired_probe(&mut self, identity: &TransferIdentity) -> DecisionOutcome {
        warn!(
            "HEAD usefulness deadline expired for {}; deferring to body",
            MediaLogIdentity::from_url(identity.source().as_str())
        );
        self.probes.require_body(identity);
        DecisionOutcome::Failed {
            class: "warp_head_probe_deadline_deferred_to_body".into(),
            elapsed_ms: 0,
        }
    }

    fn resolve_probe_claim(
        &self,
        claim: DecisionClaim,
        outcome: DecisionOutcome,
        observed_at_ms: u64,
    ) {
        self.commands
            .resolve_decision_claim(claim, outcome, observed_at_ms);
    }

    fn defer_probe_to_body(&mut self, identity: &TransferIdentity) -> bool {
        if !self.downloads.contains_identity(identity) {
            return false;
        }
        self.probes.defer_to_body(identity.post());
        true
    }
}

fn probe_observed_at(observation: &ProbeObservation) -> u64 {
    observation.outcome.as_ref().map_or_else(
        |_| crate::manager::time::unix_time_ms(),
        |result| result.observed.observed_at_ms,
    )
}

fn failure_name(class: FailureClass) -> &'static str {
    match class {
        FailureClass::Transient => "warp_head_probe_transient",
        FailureClass::Permanent => "warp_head_probe_permanent",
    }
}
