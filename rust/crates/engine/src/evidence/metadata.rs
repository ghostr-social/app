use super::{EvidenceScope, EvidenceSource, EvidenceValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NostrMetadataEvidence {
    pub issuer: String,
    pub client: Option<String>,
    pub event_id: String,
    pub observed_at_ms: u64,
    pub urls: Vec<String>,
    pub mime: Option<String>,
    pub size_bytes: Option<u64>,
    pub duration_ms: Option<u64>,
    pub dimensions: Option<(u32, u32)>,
    pub bitrate_bps: Option<u64>,
    pub sha256: Option<String>,
    pub original_sha256: Option<String>,
}

impl NostrMetadataEvidence {
    pub(crate) fn source(&self) -> EvidenceSource {
        EvidenceSource::nostr_with_client(self.issuer.clone(), self.client.clone())
    }

    pub(crate) fn scope(&self, url: &str) -> EvidenceScope {
        EvidenceScope::event_url(self.event_id.clone(), url)
    }

    pub(crate) fn values(&self) -> Vec<EvidenceValue> {
        let mut values = Vec::new();
        push(&mut values, self.mime.clone().map(EvidenceValue::Mime));
        push(&mut values, self.size_bytes.map(EvidenceValue::SizeBytes));
        push(&mut values, self.duration_ms.map(EvidenceValue::DurationMs));
        push(
            &mut values,
            self.dimensions
                .map(|(width, height)| EvidenceValue::Dimensions { width, height }),
        );
        push(&mut values, self.bitrate_bps.map(EvidenceValue::BitrateBps));
        push(
            &mut values,
            self.sha256.clone().map(EvidenceValue::AdvertisedHash),
        );
        push(
            &mut values,
            self.original_sha256
                .clone()
                .map(EvidenceValue::OriginalHash),
        );
        values
    }
}

fn push(values: &mut Vec<EvidenceValue>, value: Option<EvidenceValue>) {
    if let Some(value) = value {
        values.push(value);
    }
}
