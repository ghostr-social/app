//! Typed rendition ladders and pure adaptive quality selection.

mod policy;
mod risk;
mod types;

pub(crate) use policy::{QualitySelectionInput, QualitySelectionPolicy};
pub(crate) use types::{Rendition, RenditionId, RenditionSet};

#[cfg(test)]
#[path = "rendition_axiom_test.rs"]
pub(crate) mod axiom_test_support;
