use super::inference::merge_result;
use super::types::{
    CapabilityAttempt, CapabilityObservation, CapabilityRecord, CapabilityResult, CapabilitySignal,
    ClientCapabilityProfile, ClientCapabilityState,
};
use std::collections::VecDeque;

const ACTIVE_CAPACITY: usize = 16;
const RECORD_CAPACITY: usize = 128;
mod lookup;
mod restoration;

#[derive(Clone, Debug)]
struct ActiveTest {
    attempt: CapabilityAttempt,
    profile: ClientCapabilityProfile,
    started_us: u64,
    first_frame_recorded: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ClientCapabilityModel {
    generation: Option<u64>,
    generation_confirmed: bool,
    records: VecDeque<CapabilityRecord>,
    active: VecDeque<ActiveTest>,
    revision: u64,
}

impl ClientCapabilityModel {
    pub(crate) fn observe(&mut self, observation: CapabilityObservation) {
        match observation.event.signal {
            CapabilitySignal::Initializing => {
                self.select_generation(observation.capability_generation);
                self.start(observation);
            }
            _ if self.generation != Some(observation.capability_generation) => {}
            CapabilitySignal::Released => self.release(observation.attempt),
            signal => self.finish(&observation, signal),
        }
    }

    pub(crate) fn state(&self) -> ClientCapabilityState {
        ClientCapabilityState {
            generation: self.generation,
            records: self
                .records
                .iter()
                .filter(|record| record.profile.is_persistent())
                .cloned()
                .collect(),
            revision: self.revision,
        }
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) const fn current_generation(&self) -> Option<u64> {
        if self.generation_confirmed {
            self.generation
        } else {
            None
        }
    }

    pub(crate) fn abandon(&mut self, generation: u64, attempt: CapabilityAttempt) {
        if self.generation == Some(generation) {
            self.release(attempt);
        }
    }

    fn select_generation(&mut self, generation: u64) {
        if self.generation != Some(generation) {
            self.generation = Some(generation);
            self.records.clear();
            self.active.clear();
            self.revision = self.revision.saturating_add(1);
        }
        self.generation_confirmed = true;
    }

    fn start(&mut self, observation: CapabilityObservation) {
        self.release(observation.attempt);
        self.supersede_volatile_records(&observation.profile);
        if self.active.len() == ACTIVE_CAPACITY {
            self.active.pop_front();
        }
        self.active.push_back(ActiveTest {
            attempt: observation.attempt,
            profile: observation.profile,
            started_us: observation.event.observed_us,
            first_frame_recorded: false,
        });
    }

    fn release(&mut self, attempt: CapabilityAttempt) {
        self.active.retain(|item| item.attempt != attempt);
    }

    fn finish(&mut self, observation: &CapabilityObservation, signal: CapabilitySignal) {
        let Some(index) = self
            .active
            .iter()
            .position(|item| item.attempt == observation.attempt)
        else {
            return;
        };
        if self.active[index].profile != observation.profile {
            self.active.remove(index);
            return;
        }
        match signal {
            CapabilitySignal::FirstFrameRendered => self.record_first_frame(index, observation),
            CapabilitySignal::UnsupportedFailure | CapabilitySignal::InconclusiveFailure => {
                self.record_terminal(index, signal);
            }
            CapabilitySignal::Initializing | CapabilitySignal::Released => {}
        }
    }

    fn record_first_frame(&mut self, index: usize, observation: &CapabilityObservation) {
        let active = &mut self.active[index];
        if active.first_frame_recorded {
            return;
        }
        active.first_frame_recorded = true;
        let profile = active.profile.clone();
        let elapsed = observation
            .event
            .observed_us
            .saturating_sub(active.started_us);
        self.record(
            profile,
            CapabilityResult::Supported {
                first_frame_us: vec![elapsed],
            },
        );
    }

    fn record_terminal(&mut self, index: usize, signal: CapabilitySignal) {
        let active = self
            .active
            .remove(index)
            .expect("matched active test exists");
        let result = match signal {
            CapabilitySignal::UnsupportedFailure => CapabilityResult::Unsupported,
            CapabilitySignal::InconclusiveFailure if !active.first_frame_recorded => {
                CapabilityResult::Inconclusive
            }
            _ => return,
        };
        self.record(active.profile, result);
    }

    fn record(&mut self, profile: ClientCapabilityProfile, result: CapabilityResult) {
        if let Some(index) = self.records.iter().position(|item| item.profile == profile) {
            let mut record = self.records.remove(index).expect("matched record exists");
            record.profile.promote_persistence(profile.is_persistent());
            merge_result(&mut record.result, result);
            self.records.push_back(record);
            self.revision = self.revision.saturating_add(1);
            return;
        }
        if self.records.len() == RECORD_CAPACITY {
            self.records.pop_front();
        }
        self.records.push_back(CapabilityRecord { profile, result });
        self.revision = self.revision.saturating_add(1);
    }

    fn supersede_volatile_records(&mut self, profile: &ClientCapabilityProfile) {
        let before = self.records.len();
        self.records
            .retain(|record| !record.profile.is_superseded_by(profile));
        if self.records.len() != before {
            self.revision = self.revision.saturating_add(1);
        }
    }
}
