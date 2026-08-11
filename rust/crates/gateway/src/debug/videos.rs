//! Standalone debug-page input translated into delivery-engine focus.

use anyhow::{ensure, Context};
use ghostr_delivery::delivery_events::{DeliveryCandidate, DeliveryHandle};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use reqwest::Url;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

const MAX_URL_BYTES: usize = 8_192;
const MAX_SELECTABLE_VIDEOS: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugVideoRegistration {
    pub url: String,
    pub size_bytes: Option<u64>,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct DebugVideos {
    delivery: DeliveryHandle,
    retained: Arc<Mutex<VecDeque<DeliveryCandidate>>>,
}

impl DebugVideos {
    pub fn new(delivery: DeliveryHandle) -> Self {
        Self {
            delivery,
            retained: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn add(&self, registration: DebugVideoRegistration) -> anyhow::Result<String> {
        let candidate = delivery_candidate(registration)?;
        let id = candidate.post.as_str().to_owned();
        self.remember(candidate.clone());
        self.dispatch(candidate);
        Ok(id)
    }

    pub fn select(&self, id: &str) -> bool {
        let candidate = self
            .retained()
            .iter()
            .find(|candidate| candidate.post.as_str() == id)
            .cloned();
        let Some(candidate) = candidate else {
            return false;
        };
        self.dispatch(candidate);
        true
    }

    pub fn clear(&self) {
        self.retained().clear();
    }

    fn remember(&self, candidate: DeliveryCandidate) {
        let mut retained = self.retained();
        retained.retain(|item| item.post != candidate.post);
        if retained.len() == MAX_SELECTABLE_VIDEOS {
            retained.pop_front();
        }
        retained.push_back(candidate);
    }

    fn dispatch(&self, candidate: DeliveryCandidate) {
        let post = candidate.post.clone();
        self.delivery.admit_candidate(candidate);
        self.delivery.prioritize_candidate(post);
    }

    fn retained(&self) -> MutexGuard<'_, VecDeque<DeliveryCandidate>> {
        self.retained
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
