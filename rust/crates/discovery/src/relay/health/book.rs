use super::circuit::RelayCircuit;
use super::{
    RelayAdmission, ACTIVE_RECOVERY_PROBE_LIMIT, CIRCUIT_CAPACITY, RECOVERY_PROBES_PER_BATCH,
};
use std::collections::{HashMap, HashSet};
use tokio::time::Instant;

#[derive(Default)]
pub(super) struct HealthBook {
    circuits: HashMap<String, RelayCircuit>,
}

struct AdmissionContext<'a> {
    now: Instant,
    protected: HashSet<&'a str>,
    recovery_slots: usize,
}

impl HealthBook {
    pub(super) fn admit(&mut self, candidates: &[String], now: Instant) -> Vec<RelayAdmission> {
        let global = ACTIVE_RECOVERY_PROBE_LIMIT.saturating_sub(self.active_recoveries(now));
        let mut context = AdmissionContext {
            now,
            protected: string_set(candidates),
            recovery_slots: global.min(RECOVERY_PROBES_PER_BATCH),
        };
        candidates
            .iter()
            .filter_map(|url| self.admit_one(url, &mut context))
            .collect()
    }

    pub(super) fn observe(
        &mut self,
        admissions: &[RelayAdmission],
        completed: &[String],
        failed: &[String],
        now: Instant,
    ) {
        let completed = string_set(completed);
        let failed = string_set(failed);
        for admission in admissions {
            self.observe_one(admission, &completed, &failed, now);
        }
    }

    pub(super) fn release(&mut self, admissions: &[RelayAdmission], now: Instant) {
        for admission in admissions {
            if let Some(circuit) = self.circuits.get_mut(admission.url()) {
                circuit.release(admission, now);
            }
        }
    }

    pub(super) fn clear(&mut self) {
        self.circuits.clear();
    }

    fn admit_one(
        &mut self,
        url: &str,
        context: &mut AdmissionContext<'_>,
    ) -> Option<RelayAdmission> {
        self.ensure_room(url, context.now, &context.protected)
            .then_some(())?;
        let circuit = self.circuits.entry(url.to_owned()).or_default();
        let (generation, recovery) = circuit.admit(context.now, context.recovery_slots > 0)?;
        context.recovery_slots = context.recovery_slots.saturating_sub(usize::from(recovery));
        Some(RelayAdmission {
            url: url.to_owned(),
            generation,
        })
    }

    fn observe_one(
        &mut self,
        admission: &RelayAdmission,
        completed: &HashSet<&str>,
        failed: &HashSet<&str>,
        now: Instant,
    ) {
        let Some(circuit) = self.circuits.get_mut(admission.url()) else {
            return;
        };
        if failed.contains(admission.url()) {
            circuit.observe(admission.generation, false, now);
        } else if completed.contains(admission.url()) {
            circuit.observe(admission.generation, true, now);
        } else {
            circuit.release(admission, now);
        }
    }

    fn active_recoveries(&self, now: Instant) -> usize {
        self.circuits
            .values()
            .filter(|circuit| circuit.recovery_active(now))
            .count()
    }

    fn ensure_room(&mut self, url: &str, now: Instant, protected: &HashSet<&str>) -> bool {
        if self.circuits.contains_key(url) || self.circuits.len() < CIRCUIT_CAPACITY {
            return true;
        }
        let Some(victim) = self.eviction_candidate(now, protected) else {
            return false;
        };
        self.circuits.remove(&victim);
        true
    }

    fn eviction_candidate(&self, now: Instant, protected: &HashSet<&str>) -> Option<String> {
        self.circuits
            .iter()
            .filter(|(url, circuit)| !protected.contains(url.as_str()) && circuit.evictable(now))
            .min_by_key(|(url, circuit)| (circuit.is_cooling(), circuit.last_at, *url))
            .map(|(url, _)| url.clone())
    }
}

fn string_set(values: &[String]) -> HashSet<&str> {
    values.iter().map(String::as_str).collect()
}
