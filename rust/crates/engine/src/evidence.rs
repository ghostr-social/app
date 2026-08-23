//! Typed, conflict-preserving evidence and field-correctness calibration.

mod calibration;
mod fusion;
mod ledger;
mod metadata;
mod types;

pub use calibration::{
    CalibrationContext, CalibrationDimensions, CalibrationLabel, FieldReliabilityModel,
    ReliabilityEstimate,
};
pub use fusion::{ConfidenceAxes, EvidenceAssessment, SizeAssessment};
pub use ledger::{EvidenceInvalidation, EvidenceLedger};
pub use metadata::NostrMetadataEvidence;
pub use types::{
    Confidence, Evidence, EvidenceField, EvidenceScope, EvidenceSource, EvidenceTime,
    EvidenceValidator, EvidenceValue,
};
