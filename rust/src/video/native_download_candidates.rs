use crate::video::native_cache::NativeVideoCache;
use crate::video::native_cache_failure::is_permanent;
use crate::video::native_download_group::NativeDownloadGroup;
use crate::video::outbound_media_client::MediaHttpClient;
use log::warn;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::Instant;

const DEFAULT_CANDIDATE_LIMIT: usize = 5;
const DEFAULT_GROUP_TIMEOUT: Duration = Duration::from_secs(30);
const MINIMUM_CANDIDATE_WINDOW: Duration = Duration::from_millis(1);

#[derive(Clone, Copy)]
pub(crate) struct NativeCandidatePolicy {
    pub limit: usize,
    pub timeout: Duration,
}

impl NativeCandidatePolicy {
    pub fn new(limit: usize, timeout: Duration) -> Self {
        Self {
            limit: limit.max(1),
            timeout: timeout.max(Duration::from_millis(1)),
        }
    }
}

impl Default for NativeCandidatePolicy {
    fn default() -> Self {
        Self::new(DEFAULT_CANDIDATE_LIMIT, DEFAULT_GROUP_TIMEOUT)
    }
}

pub struct NativeDownloadOutcome {
    pub path: Option<PathBuf>,
    pub failures: Vec<NativeCandidateFailure>,
}

pub struct NativeCandidateFailure {
    pub url: String,
    pub retryable: bool,
}

struct CandidateAttempt<'a> {
    group: &'a NativeDownloadGroup,
    url: &'a str,
    timeout: Duration,
}

pub async fn download_candidates(
    cache: &NativeVideoCache,
    client: &MediaHttpClient,
    group: &NativeDownloadGroup,
    policy: NativeCandidatePolicy,
) -> NativeDownloadOutcome {
    let mut failures = Vec::new();
    let deadline = Instant::now() + policy.timeout;
    for url in group.urls.iter().take(policy.limit) {
        let Some(attempt) = candidate_attempt(group, url, deadline) else {
            break;
        };
        if let Some(outcome) = try_candidate(cache, client, attempt, &mut failures).await {
            return outcome;
        }
    }
    NativeDownloadOutcome {
        path: None,
        failures,
    }
}

fn candidate_attempt<'a>(
    group: &'a NativeDownloadGroup,
    url: &'a str,
    deadline: Instant,
) -> Option<CandidateAttempt<'a>> {
    let timeout = deadline
        .checked_duration_since(Instant::now())
        .filter(|value| *value >= MINIMUM_CANDIDATE_WINDOW)?;
    Some(CandidateAttempt {
        group,
        url,
        timeout,
    })
}

async fn try_candidate(
    cache: &NativeVideoCache,
    client: &MediaHttpClient,
    attempt: CandidateAttempt<'_>,
    failures: &mut Vec<NativeCandidateFailure>,
) -> Option<NativeDownloadOutcome> {
    let url = attempt.url;
    match download_candidate(cache, client, attempt).await {
        Ok(cached) => Some(available(cached.path)),
        Err(error) => {
            failures.push(candidate_failed(url, &error));
            None
        }
    }
}

async fn download_candidate(
    cache: &NativeVideoCache,
    client: &MediaHttpClient,
    attempt: CandidateAttempt<'_>,
) -> anyhow::Result<crate::video::native_cache::CachedVideo> {
    let request = client.get(attempt.url)?.timeout(attempt.timeout);
    cache
        .download_request(
            &attempt.group.cache_key,
            attempt.group.expected_digest.as_deref(),
            request,
        )
        .await
}

fn available(path: PathBuf) -> NativeDownloadOutcome {
    NativeDownloadOutcome {
        path: Some(path),
        failures: Vec::new(),
    }
}

fn candidate_failed(url: &str, error: &anyhow::Error) -> NativeCandidateFailure {
    warn!("Native video cache skipped a media candidate: {error}");
    NativeCandidateFailure {
        url: url.to_owned(),
        retryable: !is_permanent(error),
    }
}
