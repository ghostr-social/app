use super::{Confidence, EvidenceScope, EvidenceSource, EvidenceTime, EvidenceValidator};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum EvidenceField {
    Size,
    Mime,
    Duration,
    Dimensions,
    Bitrate,
    RangeSupport,
    FrontMoov,
    Codec,
    AdvertisedHash,
    OriginalHash,
    Readiness,
    Integrity,
}

impl EvidenceField {
    pub(crate) const ALL: [Self; 12] = [
        Self::Size,
        Self::Mime,
        Self::Duration,
        Self::Dimensions,
        Self::Bitrate,
        Self::RangeSupport,
        Self::FrontMoov,
        Self::Codec,
        Self::AdvertisedHash,
        Self::OriginalHash,
        Self::Readiness,
        Self::Integrity,
    ];

    pub(crate) fn structural(self) -> bool {
        matches!(
            self,
            Self::FrontMoov | Self::Codec | Self::Readiness | Self::Integrity
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EvidenceValue {
    SizeBytes(u64),
    Mime(String),
    DurationMs(u64),
    Dimensions { width: u32, height: u32 },
    BitrateBps(u64),
    RangeSupport(bool),
    FrontMoov(bool),
    Codec(String),
    AdvertisedHash(String),
    OriginalHash(String),
    Ready(bool),
    IntegrityMatch { digest: String, matches: bool },
}

impl EvidenceValue {
    pub(crate) fn field(&self) -> EvidenceField {
        match self {
            Self::SizeBytes(_) => EvidenceField::Size,
            Self::Mime(_) => EvidenceField::Mime,
            Self::DurationMs(_) => EvidenceField::Duration,
            Self::Dimensions { .. } => EvidenceField::Dimensions,
            Self::BitrateBps(_) => EvidenceField::Bitrate,
            Self::RangeSupport(_) => EvidenceField::RangeSupport,
            Self::FrontMoov(_) => EvidenceField::FrontMoov,
            Self::Codec(_) => EvidenceField::Codec,
            Self::AdvertisedHash(_) => EvidenceField::AdvertisedHash,
            Self::OriginalHash(_) => EvidenceField::OriginalHash,
            Self::Ready(_) => EvidenceField::Readiness,
            Self::IntegrityMatch { .. } => EvidenceField::Integrity,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Evidence<T> {
    pub(crate) value: T,
    pub(crate) source: EvidenceSource,
    pub(crate) observed_at_ms: u64,
    #[serde(default)]
    pub(crate) observed_order: u64,
    pub(crate) confidence: Confidence,
    pub(crate) validator: Option<EvidenceValidator>,
    pub(crate) scope: EvidenceScope,
    invalidated_at_ms: Option<u64>,
}

impl<T> Evidence<T> {
    pub(crate) fn new(
        value: T,
        source: EvidenceSource,
        observed_at_ms: u64,
        confidence: Confidence,
        scope: EvidenceScope,
    ) -> Self {
        let validator = scope.validator().cloned();
        Self {
            value,
            source,
            observed_at_ms,
            observed_order: 0,
            confidence,
            validator,
            scope,
            invalidated_at_ms: None,
        }
    }

    pub(crate) fn new_at(
        value: T,
        source: EvidenceSource,
        observed: EvidenceTime,
        confidence: Confidence,
        scope: EvidenceScope,
    ) -> Self {
        let mut evidence = Self::new(value, source, observed.observed_at_ms, confidence, scope);
        evidence.observed_order = observed.order;
        evidence
    }

    pub(crate) fn invalidate(&mut self, observed_at_ms: u64) -> bool {
        if self.invalidated_at_ms.is_some() {
            return false;
        }
        self.invalidated_at_ms = Some(observed_at_ms);
        true
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.invalidated_at_ms.is_none()
    }
}

#[cfg(test)]
#[path = "value/test_support.rs"]
mod test_support;
