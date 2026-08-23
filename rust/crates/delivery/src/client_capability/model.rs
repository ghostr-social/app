use super::inference::{inferred_support, merge_result, normalize_record, status_for};
use super::types::{
    CapabilityAttempt, CapabilityObservation, CapabilityRecord, CapabilityResult, CapabilitySignal,
    ClientCapabilityProfile, ClientCapabilityState, ClientCapabilityStatus,
};
use std::collections::VecDeque;

const ACTIVE_CAPACITY: usize = 16;
const RECORD_CAPACITY: usize = 128;

#[derive(Clone, Debug)]
struct ActiveTest {
    attempt: CapabilityAttempt,
    profile: ClientCapabilityProfile,
    started_us: u64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ClientCapabilityModel {
    generation: Option<u64>,
    records: VecDeque<CapabilityRecord>,
    active: VecDeque<ActiveTest>,
    revision: u64,
}

impl ClientCapabilityModel {
    pub(crate) fn observe(&mut self, observation: CapabilityObservation) {
        self.select_generation(observation.capability_generation);
        match observation.event.signal {
            CapabilitySignal::Initializing => self.start(observation),
            CapabilitySignal::Released => self.release(observation.attempt),
            signal => self.finish(observation, signal),
        }
    }

    pub(crate) fn status(
        &self,
        generation: u64,
        profile: &ClientCapabilityProfile,
    ) -> ClientCapabilityStatus {
        if self.generation != Some(generation) {
            return ClientCapabilityStatus::Unknown;
        }
        if let Some(result) = self.exact_result(profile) {
            return status_for(result);
        }
        if self.active.iter().any(|item| &item.profile == profile) {
            return ClientCapabilityStatus::Testing;
        }
        self.inferred_support(profile)
            .unwrap_or(ClientCapabilityStatus::Unknown)
    }

    #[cfg(test)]
    pub(crate) fn bounded_test_allowed(
        &self,
        generation: u64,
        profile: &ClientCapabilityProfile,
    ) -> bool {
        self.status(generation, profile) == ClientCapabilityStatus::Unknown
    }

    pub(crate) fn state(&self) -> ClientCapabilityState {
        ClientCapabilityState {
            generation: self.generation,
            records: self.records.iter().cloned().collect(),
            revision: self.revision,
        }
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) const fn current_generation(&self) -> Option<u64> {
        self.generation
    }

    pub(crate) fn abandon(&mut self, generation: u64, attempt: CapabilityAttempt) {
        if self.generation == Some(generation) {
            self.release(attempt);
        }
    }

    pub(crate) fn from_state(state: ClientCapabilityState) -> Self {
        let ClientCapabilityState {
            generation,
            records,
            revision,
        } = state;
        let Some(generation) = generation else {
            return Self::default();
        };
        let mut model = Self {
            generation: Some(generation),
            ..Self::default()
        };
        for record in records.into_iter().filter_map(normalize_record) {
            model.record(record.profile, record.result);
        }
        model.revision = model.revision.max(revision);
        model
    }

    fn select_generation(&mut self, generation: u64) {
        if self.generation != Some(generation) {
            self.generation = Some(generation);
            self.records.clear();
            self.active.clear();
            self.revision = self.revision.saturating_add(1);
        }
    }

    fn start(&mut self, observation: CapabilityObservation) {
        self.release(observation.attempt);
        if self.active.len() == ACTIVE_CAPACITY {
            self.active.pop_front();
        }
        self.active.push_back(ActiveTest {
            attempt: observation.attempt,
            profile: observation.profile,
            started_us: observation.event.observed_us,
        });
    }

    fn release(&mut self, attempt: CapabilityAttempt) {
        self.active.retain(|item| item.attempt != attempt);
    }

    fn finish(&mut self, observation: CapabilityObservation, signal: CapabilitySignal) {
        let Some(index) = self.active.iter().position(|item| {
            item.attempt == observation.attempt && item.profile == observation.profile
        }) else {
            return;
        };
        let active = self
            .active
            .remove(index)
            .expect("matched active test exists");
        let result = match signal {
            CapabilitySignal::FirstFrameRendered => CapabilityResult::Supported {
                first_frame_us: vec![observation
                    .event
                    .observed_us
                    .saturating_sub(active.started_us)],
            },
            CapabilitySignal::UnsupportedFailure => CapabilityResult::Unsupported,
            CapabilitySignal::InconclusiveFailure => CapabilityResult::Inconclusive,
            CapabilitySignal::Initializing | CapabilitySignal::Released => return,
        };
        self.record(observation.profile, result);
    }

    fn record(&mut self, profile: ClientCapabilityProfile, result: CapabilityResult) {
        if let Some(record) = self.records.iter_mut().find(|item| item.profile == profile) {
            merge_result(&mut record.result, result);
            self.revision = self.revision.saturating_add(1);
            return;
        }
        if self.records.len() == RECORD_CAPACITY {
            self.records.pop_front();
        }
        self.records.push_back(CapabilityRecord { profile, result });
        self.revision = self.revision.saturating_add(1);
    }

    fn exact_result(&self, profile: &ClientCapabilityProfile) -> Option<&CapabilityResult> {
        self.records
            .iter()
            .find(|item| &item.profile == profile)
            .map(|item| &item.result)
    }

    fn inferred_support(
        &self,
        profile: &ClientCapabilityProfile,
    ) -> Option<ClientCapabilityStatus> {
        inferred_support(&self.records, profile)
    }
}
