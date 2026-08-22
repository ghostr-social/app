use serde::{Deserialize, Serialize};

const BUCKETS: usize = 10;
const HALF_LIFE_MS: u64 = 30 * 24 * 60 * 60 * 1_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct CalibrationState {
    counts: Vec<f64>,
    positives: Vec<f64>,
    predicted: Vec<f64>,
    last_updated_ms: u64,
    labels: u64,
}

impl Default for CalibrationState {
    fn default() -> Self {
        Self {
            counts: vec![0.0; BUCKETS],
            positives: vec![0.0; BUCKETS],
            predicted: vec![0.0; BUCKETS],
            last_updated_ms: 0,
            labels: 0,
        }
    }
}

impl CalibrationState {
    pub(crate) fn observe(&mut self, prediction: f64, positive: bool, now_ms: u64) {
        self.decay(now_ms);
        let bucket = bucket(prediction);
        self.counts[bucket] += 1.0;
        self.positives[bucket] += f64::from(positive);
        self.predicted[bucket] += prediction;
        self.labels = self.labels.saturating_add(1);
    }

    pub(crate) fn calibrate(&self, prediction: f64, now_ms: u64) -> f64 {
        let bucket = bucket(prediction);
        let scale = decay_scale(now_ms, self.last_updated_ms);
        let count = self.counts[bucket] * scale;
        if count == 0.0 {
            return prediction;
        }
        let empirical = self.positives[bucket] * scale / count;
        let confidence = count / (count + 8.0);
        (prediction * (1.0 - confidence) + empirical * confidence).clamp(0.0, 1.0)
    }

    pub(crate) fn error_bps(&self) -> u16 {
        let total: f64 = self.counts.iter().sum();
        if total == 0.0 {
            return 0;
        }
        let error = (0..BUCKETS)
            .map(|index| self.bucket_error(index))
            .sum::<f64>()
            / total;
        (error.clamp(0.0, 1.0) * 10_000.0).round() as u16
    }

    pub(crate) fn labels(&self) -> u64 {
        self.labels
    }

    pub(crate) fn sanitize(mut self) -> Self {
        sanitize(&mut self.counts);
        sanitize(&mut self.positives);
        sanitize(&mut self.predicted);
        self
    }

    fn bucket_error(&self, index: usize) -> f64 {
        let count = self.counts[index];
        if count == 0.0 {
            return 0.0;
        }
        (self.positives[index] / count - self.predicted[index] / count).abs() * count
    }

    fn decay(&mut self, now_ms: u64) {
        let scale = decay_scale(now_ms, self.last_updated_ms);
        self.counts.iter_mut().for_each(|value| *value *= scale);
        self.positives.iter_mut().for_each(|value| *value *= scale);
        self.predicted.iter_mut().for_each(|value| *value *= scale);
        self.last_updated_ms = now_ms;
    }
}

fn bucket(probability: f64) -> usize {
    (probability.clamp(0.0, 1.0) * BUCKETS as f64)
        .floor()
        .min((BUCKETS - 1) as f64) as usize
}

fn decay_scale(now_ms: u64, then_ms: u64) -> f64 {
    if then_ms == 0 || now_ms <= then_ms {
        return 1.0;
    }
    0.5_f64.powf(now_ms.saturating_sub(then_ms) as f64 / HALF_LIFE_MS as f64)
}

fn sanitize(values: &mut Vec<f64>) {
    values.resize(BUCKETS, 0.0);
    values.truncate(BUCKETS);
    values.iter_mut().for_each(|value| {
        if !value.is_finite() || *value < 0.0 {
            *value = 0.0;
        }
    });
}
