use super::{hedge, ChunkAttempt, CompletionStatus, InFlightChunks};

#[cfg(test)]
#[path = "network_reservation/exploration_admission_test.rs"]
mod exploration_admission_test;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NetworkReservation {
    committed_bytes: u64,
    uncommitted_prefix_bytes: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FinishedAction {
    status: CompletionStatus,
    network_reservation: Option<NetworkReservation>,
    admission_claim: Option<ghostr_engine::origin_model::AdmissionClaim>,
}

impl NetworkReservation {
    pub(crate) const fn actual_bytes(self, received_bytes: u64) -> u64 {
        received_bytes.saturating_sub(self.uncommitted_prefix_bytes)
    }

    pub(crate) const fn committed_bytes(self) -> u64 {
        self.committed_bytes
    }
}

impl FinishedAction {
    pub(crate) const fn status(&self) -> CompletionStatus {
        self.status
    }

    pub(crate) const fn network_reservation(&self) -> Option<NetworkReservation> {
        self.network_reservation
    }

    pub(crate) fn exploration_admitted(&self) -> bool {
        self.admission_claim
            .as_ref()
            .is_some_and(ghostr_engine::origin_model::AdmissionClaim::is_exploration)
    }

    pub(crate) fn take_admission_claim(
        &mut self,
    ) -> Option<ghostr_engine::origin_model::AdmissionClaim> {
        self.admission_claim.take()
    }
}

impl InFlightChunks {
    pub(crate) fn finish_with_resources(&mut self, attempt: &ChunkAttempt) -> FinishedAction {
        let Some(active) = self.transfers.get(&attempt.id()) else {
            return superseded();
        };
        if active.identity != *attempt.identity() || active.chunk != attempt.chunk {
            return superseded();
        }
        let status = hedge::completion_status(active, self.hedges.contains_key(&attempt.id()));
        let network_reservation =
            (active.committed_network_bytes > 0).then_some(NetworkReservation {
                committed_bytes: active.committed_network_bytes,
                uncommitted_prefix_bytes: active.uncommitted_network_prefix_bytes,
            });
        let active = self
            .transfers
            .remove(&attempt.id())
            .expect("validated active transfer");
        self.hedges.remove(&attempt.id());
        FinishedAction {
            status,
            network_reservation,
            admission_claim: active.admission_claim,
        }
    }
}

const fn superseded() -> FinishedAction {
    FinishedAction {
        status: CompletionStatus::Superseded,
        network_reservation: None,
        admission_claim: None,
    }
}
