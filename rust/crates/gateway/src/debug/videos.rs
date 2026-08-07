//! Standalone debug-page input translated into delivery-engine focus.

use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_delivery::delivery_events::{DeliveryCandidate, DeliveryHandle};
use anyhow::{ensure, Context};
use reqwest::Url;
use sha2::{Digest, Sha256};

const MAX_URL_BYTES: usize = 8_192;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugVideoRegistration {
    pub url: String,
    pub size_bytes: Option<u64>,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct DebugVideos {
    delivery: DeliveryHandle,
}

impl DebugVideos {
    pub fn new(delivery: DeliveryHandle) -> Self {
        Self { delivery }
    }

    pub fn add(&self, registration: DebugVideoRegistration) -> anyhow::Result<String> {
        let candidate = delivery_candidate(registration)?;
        let id = candidate.post.as_str().to_owned();
        self.delivery.admit_candidate(candidate.clone());
        self.delivery.prioritize_candidate(candidate.post);
        Ok(id)
    }
}

fn delivery_candidate(registration: DebugVideoRegistration) -> anyhow::Result<DeliveryCandidate> {
    validate_measurement(registration.size_bytes, "size")?;
    validate_measurement(registration.duration_ms, "duration")?;
    let url = validated_url(registration.url)?;
    Ok(DeliveryCandidate {
        post: PostId::new(debug_id(&url)),
        meta: VideoMeta {
            urls: vec![url],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: registration.size_bytes,
            duration_ms: registration.duration_ms,
        },
        discovered_at: 0,
    })
}

fn validated_url(raw: String) -> anyhow::Result<String> {
    let raw = raw.trim();
    ensure!(!raw.is_empty(), "video URL is required");
    ensure!(raw.len() <= MAX_URL_BYTES, "video URL is too long");
    let parsed = Url::parse(raw).context("video URL is invalid")?;
    ensure!(
        matches!(parsed.scheme(), "http" | "https") && parsed.host().is_some(),
        "video URL must use HTTP or HTTPS"
    );
    Ok(parsed.to_string())
}

fn validate_measurement(value: Option<u64>, name: &str) -> anyhow::Result<()> {
    ensure!(value != Some(0), "{name} must be positive");
    Ok(())
}

fn debug_id(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    let suffix: String = digest[..12]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("debug-{suffix}")
}
