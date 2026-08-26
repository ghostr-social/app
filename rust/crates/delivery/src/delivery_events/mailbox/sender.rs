use super::{control, lock, signal, MailboxSender, MailboxState};
use crate::delivery_events::{
    CandidateAdmission, DeliveryCandidate, DeliveryCommand, DeliveryFocus, FocusAdmission,
};
use std::sync::MutexGuard;

#[cfg(test)]
mod generated_focus_api_test;

impl MailboxSender {
    pub(in crate::delivery_events) fn send_control(&self, command: DeliveryCommand) -> bool {
        if self.control_wake.is_closed() {
            return false;
        }
        control::replace(&mut self.lock().controls, command);
        signal(&self.control_wake)
    }

    pub(in crate::delivery_events) fn send_focus(&self, focus: DeliveryFocus) -> FocusAdmission {
        if self.control_wake.is_closed() {
            return FocusAdmission::Closed;
        }
        let mut state = self.lock();
        if !state.focus_generations.accept(focus.generation) {
            return FocusAdmission::Stale;
        }
        control::replace(&mut state.controls, DeliveryCommand::Focus(focus));
        drop(state);
        if signal(&self.control_wake) {
            FocusAdmission::Accepted
        } else {
            FocusAdmission::Closed
        }
    }

    pub(in crate::delivery_events) fn send_network_profile(
        &self,
        profile: crate::debug::network::NetworkProfile,
    ) -> Option<u64> {
        if self.control_wake.is_closed() {
            return None;
        }
        let mut state = self.lock();
        let generation = state.next_network_profile_generation.checked_add(1)?;
        state.next_network_profile_generation = generation;
        control::replace(
            &mut state.controls,
            DeliveryCommand::NetworkProfile {
                generation,
                profile,
            },
        );
        drop(state);
        signal(&self.control_wake).then_some(generation)
    }

    pub(in crate::delivery_events) fn send_candidate(
        &self,
        candidate: DeliveryCandidate,
    ) -> CandidateAdmission {
        if self.candidate_wake.is_closed() {
            return CandidateAdmission::Closed;
        }
        let mut state = self.lock();
        if state.candidates.len() >= state.candidate_capacity {
            return CandidateAdmission::Saturated;
        }
        state.candidates.push_back(candidate);
        drop(state);
        if signal(&self.candidate_wake) {
            CandidateAdmission::Accepted
        } else {
            CandidateAdmission::Closed
        }
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, MailboxState> {
        lock(&self.state)
    }
}
