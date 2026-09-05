use super::Catalog;
use crate::representation::RepresentationGeneration;
use std::collections::{BTreeSet, HashMap};

impl Default for Catalog {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            reliability: crate::evidence::FieldReliabilityModel::default(),
            reliability_revision: 0,
            quarantined_sources: BTreeSet::new(),
            next_generation: RepresentationGeneration::first(),
        }
    }
}
