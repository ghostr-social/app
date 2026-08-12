use super::ConcurrencyEvidence;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct EvidenceWindow {
    throughput_sum: u128,
    ttfb_micros_sum: u128,
    count: usize,
}

impl EvidenceWindow {
    pub(super) fn push(&mut self, evidence: ConcurrencyEvidence) {
        self.throughput_sum = self
            .throughput_sum
            .saturating_add(u128::from(evidence.aggregate_bytes_per_second));
        self.ttfb_micros_sum = self
            .ttfb_micros_sum
            .saturating_add(evidence.ttfb.as_micros());
        self.count = self.count.saturating_add(1);
    }

    pub(super) fn len(self) -> usize {
        self.count
    }

    pub(super) fn throughput(self) -> u64 {
        average(self.throughput_sum, self.count)
    }

    pub(super) fn ttfb_micros(self) -> u64 {
        average(self.ttfb_micros_sum, self.count)
    }
}

fn average(total: u128, count: usize) -> u64 {
    (total / count.max(1) as u128).min(u128::from(u64::MAX)) as u64
}
