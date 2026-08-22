use super::{
    CandidateSerde, CapabilitySerde, PreviewSerde, QualitySerde, RetrySerde, TransformSerde,
};
use crate::adaptive::{
    HeadProbeHistory, PlannerCandidateContext, PlannerCapability, PlannerQuality,
    PlannerRetryAvailability, PreviewAvailability, TransformCapability,
};

impl From<PlannerCandidateContext> for CandidateSerde {
    fn from(value: PlannerCandidateContext) -> Self {
        Self {
            semantic: value.semantic,
            capability: value.capability.into(),
            quality: value.quality.into(),
            preview: value.preview.into(),
            watch: value.watch,
            head_probe: head_code(value.head_probe),
            retry: value.retry.into(),
        }
    }
}

impl From<CandidateSerde> for PlannerCandidateContext {
    fn from(value: CandidateSerde) -> Self {
        Self {
            semantic: value.semantic,
            capability: value.capability.into(),
            quality: value.quality.into(),
            preview: value.preview.into(),
            watch: value.watch,
            head_probe: head(value.head_probe),
            retry: value.retry.into(),
        }
    }
}

impl From<PlannerCapability> for CapabilitySerde {
    fn from(value: PlannerCapability) -> Self {
        match value {
            PlannerCapability::Unavailable => Self::Unavailable,
            PlannerCapability::Reported {
                playback_supported,
                transform,
                evidence_epoch,
            } => Self::Reported {
                playback_supported,
                transform: transform.map(Into::into),
                evidence_epoch,
            },
        }
    }
}

impl From<CapabilitySerde> for PlannerCapability {
    fn from(value: CapabilitySerde) -> Self {
        match value {
            CapabilitySerde::Unavailable => Self::Unavailable,
            CapabilitySerde::Reported {
                playback_supported,
                transform,
                evidence_epoch,
            } => Self::Reported {
                playback_supported,
                transform: transform.map(Into::into),
                evidence_epoch,
            },
        }
    }
}

impl From<TransformCapability> for TransformSerde {
    fn from(value: TransformCapability) -> Self {
        Self {
            kind: value.kind,
            estimated_cpu_ms: value.estimated_cpu_ms,
            output_upper_bytes: value.output_upper_bytes,
        }
    }
}

impl From<TransformSerde> for TransformCapability {
    fn from(value: TransformSerde) -> Self {
        Self::new(value.kind, value.estimated_cpu_ms, value.output_upper_bytes)
    }
}

impl From<PlannerQuality> for QualitySerde {
    fn from(value: PlannerQuality) -> Self {
        match value {
            PlannerQuality::Unavailable => Self::Unavailable,
            PlannerQuality::Estimated {
                expected_micros,
                lower_micros,
                uncertainty_bps,
            } => Self::Estimated {
                expected_micros,
                lower_micros,
                uncertainty_bps,
            },
        }
    }
}

impl From<QualitySerde> for PlannerQuality {
    fn from(value: QualitySerde) -> Self {
        match value {
            QualitySerde::Unavailable => Self::Unavailable,
            QualitySerde::Estimated {
                expected_micros,
                lower_micros,
                uncertainty_bps,
            } => Self::Estimated {
                expected_micros,
                lower_micros,
                uncertainty_bps,
            },
        }
    }
}

impl From<PreviewAvailability> for PreviewSerde {
    fn from(value: PreviewAvailability) -> Self {
        match value {
            PreviewAvailability::Unavailable => Self::Unavailable,
            PreviewAvailability::Ready {
                bytes,
                quality_micros,
            } => Self::Ready {
                bytes,
                quality_micros,
            },
        }
    }
}

impl From<PreviewSerde> for PreviewAvailability {
    fn from(value: PreviewSerde) -> Self {
        match value {
            PreviewSerde::Unavailable => Self::Unavailable,
            PreviewSerde::Ready {
                bytes,
                quality_micros,
            } => Self::Ready {
                bytes,
                quality_micros,
            },
        }
    }
}

impl From<PlannerRetryAvailability> for RetrySerde {
    fn from(value: PlannerRetryAvailability) -> Self {
        match value {
            PlannerRetryAvailability::Ready => Self::Ready,
            PlannerRetryAvailability::Cooling { eligible_at_ms } => {
                Self::Cooling { eligible_at_ms }
            }
        }
    }
}

impl From<RetrySerde> for PlannerRetryAvailability {
    fn from(value: RetrySerde) -> Self {
        match value {
            RetrySerde::Ready => Self::Ready,
            RetrySerde::Cooling { eligible_at_ms } => Self::Cooling { eligible_at_ms },
        }
    }
}

const fn head_code(value: HeadProbeHistory) -> u8 {
    match value {
        HeadProbeHistory::Unobserved => 0,
        HeadProbeHistory::Active => 1,
        HeadProbeHistory::Completed => 2,
    }
}

const fn head(value: u8) -> HeadProbeHistory {
    match value {
        1 => HeadProbeHistory::Active,
        2 => HeadProbeHistory::Completed,
        _ => HeadProbeHistory::Unobserved,
    }
}
