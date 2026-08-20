use super::Catalog;
use crate::evidence::{Confidence, Evidence, EvidenceSource, EvidenceValue};
use crate::media_timeline::MediaTimeline;
use crate::representation::RepresentationBinding;

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

    pub fn learn_timeline_observation_for(
        &mut self,
        binding: &RepresentationBinding,
        timeline: MediaTimeline,
        observed_at_ms: u64,
    ) -> bool {
        let Some(entry) = self.current_entry(binding) else {
            return false;
        };
        let labels = entry.record_parser_evidence(&timeline, observed_at_ms);
        entry.timeline = Some(timeline);
        entry.tail_timeline_needed = false;
        entry.evidence_clock_ms = entry.evidence_clock_ms.max(observed_at_ms);
        self.observe_labels(labels, observed_at_ms);
        true
    }

    /// Records that a bounded head inspection found no timing metadata.
    pub fn require_tail_timeline_for(&mut self, binding: &RepresentationBinding) -> bool {
        let Some(entry) = self.current_entry(binding) else {
            return false;
        };
        entry.tail_timeline_needed = true;
        true
    }

    /// Drops structural offsets when the stored byte generation changes.
    pub fn clear_timeline_for(&mut self, binding: &RepresentationBinding) -> bool {
        let Some(entry) = self.current_entry(binding) else {
            return false;
        };
        let observed_at_ms = entry.evidence_clock_ms.saturating_add(1);
        entry.ledger.invalidate_parser(observed_at_ms);
        entry.evidence_clock_ms = observed_at_ms;
        entry.timeline = None;
        entry.tail_timeline_needed = false;
        true
    }

    fn current_entry(
        &mut self,
        binding: &RepresentationBinding,
    ) -> Option<&mut super::CatalogEntry> {
        let entry = self.entries.get_mut(binding.post())?;
        (&entry.binding == binding).then_some(entry)
    }
}

impl super::CatalogEntry {
    fn record_parser_evidence(
        &mut self,
        timeline: &MediaTimeline,
        observed_at_ms: u64,
    ) -> Vec<crate::evidence::CalibrationLabel> {
        let source = EvidenceSource::parser("mp4-v3");
        let truths = parser_values(timeline);
        let mut labels = Vec::new();
        for url in self.meta.urls.clone() {
            labels.extend(self.calibration_labels(&url, &truths, &source, observed_at_ms));
            let scope = self.ledger.scope_for_url(&url);
            for value in &truths {
                self.ledger.record(Evidence::new(
                    value.clone(),
                    source.clone(),
                    observed_at_ms,
                    Confidence::certain(),
                    scope.clone(),
                ));
            }
        }
        labels
    }
}

fn parser_values(timeline: &MediaTimeline) -> Vec<EvidenceValue> {
    let mut values = vec![
        EvidenceValue::FrontMoov(timeline.front_moov()),
        EvidenceValue::Ready(timeline.startup_footprint().is_some()),
    ];
    if let Some(duration) = timeline.duration_ms() {
        values.push(EvidenceValue::DurationMs(duration));
    }
    values
}
