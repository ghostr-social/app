use super::Catalog;
use crate::evidence::{Confidence, Evidence, EvidenceScope, EvidenceSource, EvidenceValue};
use crate::representation::RepresentationBinding;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackEvidence {
    pub client: String,
    pub first_frame: bool,
    pub observed_at_ms: u64,
}

impl PlaybackEvidence {
    pub fn new(client: impl Into<String>, first_frame: bool, observed_at_ms: u64) -> Self {
        Self {
            client: client.into(),
            first_frame,
            observed_at_ms,
        }
    }
}

impl Catalog {
    pub fn learn_playback_for(
        &mut self,
        binding: &RepresentationBinding,
        observation: PlaybackEvidence,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(binding.post()) else {
            return false;
        };
        if &entry.binding != binding {
            return false;
        }
        let source = EvidenceSource::playback(observation.client.clone());
        let truth = EvidenceValue::Ready(observation.first_frame);
        let labels = entry
            .meta
            .urls
            .iter()
            .flat_map(|url| {
                entry.calibration_labels(
                    url,
                    std::slice::from_ref(&truth),
                    &source,
                    observation.observed_at_ms,
                )
            })
            .collect();
        entry.ledger.record(Evidence::new(
            truth,
            source,
            observation.observed_at_ms,
            Confidence::new(9_500).unwrap(),
            EvidenceScope::ClientVersion(observation.client),
        ));
        entry.evidence_clock_ms = entry.evidence_clock_ms.max(observation.observed_at_ms);
        self.observe_labels(labels, observation.observed_at_ms);
        true
    }
}
