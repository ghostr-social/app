use super::*;

impl Catalog {
    /// Installs timing only for the representation that produced the bytes.
    pub fn learn_timeline_for(
        &mut self,
        binding: &RepresentationBinding,
        timeline: MediaTimeline,
    ) -> bool {
        let observed_at_ms = self
            .entries
            .get(binding.post())
            .map_or(1, |entry| entry.evidence_clock_ms.saturating_add(1));
        self.learn_timeline_observation_for(binding, timeline, observed_at_ms)
    }
}
