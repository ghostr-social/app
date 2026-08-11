//! Typed rendition ladders and pure adaptive quality selection.

mod policy;
mod risk;
mod types;

pub use policy::{
    DowngradeCause, QualityChange, QualityDecision, QualitySelectionInput, QualitySelectionPolicy,
};
pub use types::{Rendition, RenditionError, RenditionId, RenditionSet, RenditionSetError};
