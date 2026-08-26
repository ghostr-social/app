use serde::{Deserialize, Serialize};

pub(crate) const BINS: usize = 100;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct HazardStats {
    risk: Vec<f64>,
    events: Vec<f64>,
    samples: f64,
    last_updated_ms: u64,
    pub(super) last_used_ms: u64,
}

impl Default for HazardStats {
    fn default() -> Self {
        Self {
            risk: vec![0.0; BINS],
            events: vec![0.0; BINS],
            samples: 0.0,
            last_updated_ms: 0,
            last_used_ms: 0,
        }
    }
}

impl HazardStats {
    pub(super) fn observe(&mut self, watched_ms: u64, event: bool, now_ms: u64, half_life: u64) {
        self.decay_in_place(now_ms, half_life);
        for index in 0..BINS {
            if bin_start_ms(index) > watched_ms {
                break;
            }
            self.risk[index] += 1.0;
        }
        if event {
            self.events[bin_index(watched_ms)] += 1.0;
        }
        self.samples += 1.0;
        self.last_used_ms = now_ms;
    }

    pub(super) fn effective_samples(&self, now_ms: u64, half_life: u64) -> f64 {
        self.samples * decay(now_ms, self.last_updated_ms, half_life)
    }

    pub(super) fn survival(&self, at_ms: u64, now_ms: u64, half_life: u64) -> f64 {
        let scale = decay(now_ms, self.last_updated_ms, half_life);
        let mut survival = 1.0;
        for index in 0..=bin_index(at_ms) {
            let prior = cold_hazard(index);
            let risk = self.risk[index] * scale;
            let events = self.events[index] * scale;
            let hazard = (events + prior * 2.0) / (risk + 2.0);
            survival *= 1.0 - hazard.clamp(0.0, 1.0);
        }
        survival.clamp(0.0, 1.0)
    }

    pub(super) fn sanitize(mut self) -> Self {
        self.risk.resize(BINS, 0.0);
        self.events.resize(BINS, 0.0);
        self.risk.truncate(BINS);
        self.events.truncate(BINS);
        sanitize_values(&mut self.risk);
        sanitize_values(&mut self.events);
        self.samples = finite(self.samples);
        self
    }

    fn decay_in_place(&mut self, now_ms: u64, half_life: u64) {
        let scale = decay(now_ms, self.last_updated_ms, half_life);
        if scale < 1.0 {
            self.risk.iter_mut().for_each(|value| *value *= scale);
            self.events.iter_mut().for_each(|value| *value *= scale);
            self.samples *= scale;
        }
        self.last_updated_ms = now_ms;
    }
}

pub(crate) fn bin_end_ms(index: usize) -> u64 {
    match index {
        0..=19 => (index as u64 + 1) * 250,
        20..=44 => 5_000 + (index as u64 - 19) * 1_000,
        45..=74 => 30_000 + (index as u64 - 44) * 5_000,
        _ => 180_000 + (index as u64 - 74) * 30_000,
    }
}

fn bin_start_ms(index: usize) -> u64 {
    index.checked_sub(1).map_or(0, bin_end_ms)
}

fn bin_index(at_ms: u64) -> usize {
    (0..BINS)
        .find(|index| at_ms < bin_end_ms(*index))
        .unwrap_or(BINS - 1)
}

fn cold_hazard(index: usize) -> f64 {
    let start = bin_start_ms(index);
    let end = bin_end_ms(index);
    let start_survival = cold_survival(start);
    1.0 - cold_survival(end) / start_survival.max(f64::MIN_POSITIVE)
}

pub(super) fn cold_survival(at_ms: u64) -> f64 {
    let seconds = at_ms as f64 / 1_000.0;
    0.55 * (-seconds / 1.5).exp() + 0.45 * (-seconds / 18.0).exp()
}

fn decay(now_ms: u64, then_ms: u64, half_life: u64) -> f64 {
    if then_ms == 0 || now_ms <= then_ms {
        return 1.0;
    }
    0.5_f64.powf(now_ms.saturating_sub(then_ms) as f64 / half_life as f64)
}

fn sanitize_values(values: &mut [f64]) {
    for value in values.iter_mut() {
        *value = finite(*value);
    }
}

fn finite(value: f64) -> f64 {
    if value.is_finite() && value >= 0.0 {
        value.min(1_000_000.0)
    } else {
        0.0
    }
}
