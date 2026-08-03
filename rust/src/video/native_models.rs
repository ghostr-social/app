use crate::video::native_candidate_round::NativeCandidateRound;
use crate::video::native_download_state::NativeDownloadState;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct NativeUserData {
    pub npub: Option<String>,
    pub name: Option<String>,
    pub profile_picture: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NativeEventIdentity {
    pub event_id: String,
    pub author_public_key_hex: String,
    pub kind: u16,
    pub identifier: Option<String>,
    pub created_at: u64,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct NativeVideo {
    pub id: String,
    pub expected_digest: Option<String>,
    pub fallback_urls: Vec<String>,
    pub user: NativeUserData,
    pub title: String,
    pub song_name: String,
    pub comments: String,
    pub likes: String,
    pub url: String,
    pub delivery: NativeVideoDelivery,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NativeVideoCacheKey {
    AdvertisedDigest(String),
    UrlDerived(String),
}

impl NativeVideoCacheKey {
    pub fn storage_id(&self) -> Option<String> {
        let (namespace, value) = match self {
            Self::AdvertisedDigest(value) => ("digest", value),
            Self::UrlDerived(value) => ("url", value),
        };
        if value.len() != 64 || !value.chars().all(|item| item.is_ascii_hexdigit()) {
            return None;
        }
        Some(format!(
            "{:x}",
            Sha256::digest(format!("{namespace}\0{value}").as_bytes())
        ))
    }
}

impl NativeVideo {
    pub fn cache_key(&self) -> NativeVideoCacheKey {
        match &self.expected_digest {
            Some(digest) => NativeVideoCacheKey::AdvertisedDigest(digest.clone()),
            None => NativeVideoCacheKey::UrlDerived(self.id.clone()),
        }
    }

    pub fn source_urls(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.url.as_str()).chain(self.fallback_urls.iter().map(String::as_str))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeVideoDelivery {
    Progressive,
    Hls,
}

impl NativeVideoDelivery {
    pub fn can_cache_as_single_file(self) -> bool {
        self == Self::Progressive
    }
}

#[derive(Clone, Debug)]
pub struct NativeVideoDownload {
    pub id: String,
    pub url: String,
    pub event: NativeEventIdentity,
    pub nostr: NativeVideo,
    candidate_round: NativeCandidateRound,
    state: NativeDownloadState,
    suppressed_by: Option<NativeVideoCacheKey>,
}

impl NativeVideoDownload {
    pub fn new(inventory_id: String, nostr: NativeVideo, event: NativeEventIdentity) -> Self {
        let state = NativeDownloadState::initial(nostr.delivery.can_cache_as_single_file());
        Self {
            id: inventory_id,
            url: nostr.url.clone(),
            event,
            nostr,
            candidate_round: NativeCandidateRound::default(),
            state,
            suppressed_by: None,
        }
    }

    pub fn is_downloading(&self) -> bool {
        self.state.is_downloading()
    }

    pub fn participates_in_cache(&self) -> bool {
        self.nostr.delivery.can_cache_as_single_file()
    }

    pub fn local_path(&self) -> Option<&Path> {
        self.state.local_path()
    }

    pub fn begin_retry(&mut self, now: tokio::time::Instant) -> bool {
        let due = self.state.begin_retry(now);
        if due {
            self.reset_candidate_round();
        }
        due
    }

    pub fn finish_download(&mut self, path: Option<PathBuf>, retryable: bool) {
        self.state
            .finish(path, retryable, tokio::time::Instant::now());
    }

    pub fn mark_available(&mut self, path: PathBuf) {
        self.suppressed_by = None;
        self.reset_candidate_round();
        self.state.mark_available(path);
    }

    pub fn restart_download(&mut self) {
        self.suppressed_by = None;
        self.reset_candidate_round();
        self.state.restart();
    }

    pub(crate) fn pending_source_urls(&self) -> impl Iterator<Item = &str> {
        self.nostr
            .source_urls()
            .filter(|url| self.candidate_round.is_pending(url))
    }

    pub(crate) fn record_candidate_failure(&mut self, url: &str, retryable: bool) -> bool {
        if !self.nostr.source_urls().any(|candidate| candidate == url) {
            return false;
        }
        self.candidate_round.record_failure(url, retryable);
        true
    }

    pub(crate) fn finish_candidate_round_if_exhausted(&mut self) {
        if self.pending_source_urls().next().is_none() {
            self.finish_download(None, self.candidate_round.is_retryable());
        }
    }

    pub fn suppress(&mut self, by: NativeVideoCacheKey) {
        self.suppressed_by = Some(by);
        self.state.suppress();
    }

    pub fn suppressed_by(&self) -> Option<&NativeVideoCacheKey> {
        self.suppressed_by.as_ref()
    }

    pub fn is_rejected(&self) -> bool {
        self.state.is_rejected()
    }

    fn reset_candidate_round(&mut self) {
        self.candidate_round.reset();
    }
}

pub type NativeDownloads = Arc<Mutex<HashMap<String, NativeVideoDownload>>>;

pub fn new_native_downloads() -> NativeDownloads {
    Arc::new(Mutex::new(HashMap::new()))
}
