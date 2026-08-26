use super::*;

impl CalibrationState {
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

    fn bucket_error(&self, index: usize) -> f64 {
        let count = self.counts[index];
        if count == 0.0 {
            return 0.0;
        }
        (self.positives[index] / count - self.predicted[index] / count).abs() * count
    }
}
