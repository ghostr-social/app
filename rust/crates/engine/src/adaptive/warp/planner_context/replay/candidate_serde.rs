mod conversions;

use super::super::PlannerCandidateContext;
use crate::adaptive::{PlannerWatchEvidence, SemanticScore, TransformKind};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Deserialize, Serialize)]
struct CandidateSerde {
    semantic: SemanticScore,
    capability: CapabilitySerde,
    quality: QualitySerde,
    preview: PreviewSerde,
    watch: PlannerWatchEvidence,
    head_probe: u8,
    retry: RetrySerde,
}

#[derive(Deserialize, Serialize)]
enum CapabilitySerde {
    Unavailable,
    Reported {
        playback_supported: bool,
        transform: Option<TransformSerde>,
        evidence_epoch: u64,
    },
}

#[derive(Clone, Copy, Deserialize, Serialize)]
struct TransformSerde {
    kind: TransformKind,
    estimated_cpu_ms: u64,
    output_upper_bytes: u64,
}

#[derive(Deserialize, Serialize)]
enum QualitySerde {
    Unavailable,
    Estimated {
        expected_micros: u64,
        lower_micros: u64,
        uncertainty_bps: u16,
    },
}

#[derive(Deserialize, Serialize)]
enum PreviewSerde {
    Unavailable,
    Ready { bytes: u64, quality_micros: u64 },
}

#[derive(Deserialize, Serialize)]
enum RetrySerde {
    Ready,
    Cooling { eligible_at_ms: u64 },
}

impl Serialize for PlannerCandidateContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        CandidateSerde::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PlannerCandidateContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(CandidateSerde::deserialize(deserializer)?.into())
    }
}
