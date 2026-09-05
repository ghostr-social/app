use super::source_rejection::{SourceRejection, MAX_REJECTIONS};
use super::Catalog;
use crate::evidence::FieldReliabilityModel;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CatalogEvidenceState {
    reliability: FieldReliabilityModel,
    quarantined_sources: BTreeSet<SourceRejection>,
}

impl CatalogEvidenceState {
    pub fn from_reliability(reliability: FieldReliabilityModel) -> Self {
        Self {
            reliability,
            ..Self::default()
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("catalog evidence always serializes")
    }

    /// Restores persisted catalog evidence.
    ///
    /// # Errors
    ///
    /// Returns a JSON decoding error when the persisted state is malformed.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut state: Self = serde_json::from_str(json)?;
        state.reliability = FieldReliabilityModel::normalized(state.reliability);
        state.quarantined_sources = state
            .quarantined_sources
            .into_iter()
            .take(MAX_REJECTIONS)
            .collect();
        Ok(state)
    }
}

impl Catalog {
    pub fn evidence_state(&self) -> CatalogEvidenceState {
        CatalogEvidenceState {
            reliability: self.reliability.clone(),
            quarantined_sources: self.quarantined_sources.clone(),
        }
    }

    pub fn replace_evidence_state(&mut self, state: CatalogEvidenceState, now_ms: u64) {
        self.reliability = state.reliability;
        self.quarantined_sources = state.quarantined_sources;
        self.recalibrate(now_ms);
        let posts: Vec<_> = self.entries.keys().cloned().collect();
        for post in posts {
            self.apply_known_quarantine(&post);
        }
    }
}
