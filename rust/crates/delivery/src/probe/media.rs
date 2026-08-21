//! HEAD probe service (plan Phase 1 step 5): learns a video's size,
//! range support, and content type through the SSRF-safe outbound
//! client. Body requests remain exclusively owned by policy grants.

use anyhow::{Context, Result};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::RequestAuthority;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};
use ghostr_net::origin_content_type;
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use reqwest::header::{
    HeaderName, HeaderValue, ACCEPT_ENCODING, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE, ETAG,
    LAST_MODIFIED,
};
use std::future::Future;
use std::time::Duration;
use tokio::time::Instant;

/// What one probe learned about a media URL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeResult {
    pub content_length: Option<u64>,
    pub accept_ranges: Option<bool>,
    pub content_type: Option<String>,
    pub validator: Option<EvidenceValidator>,
    pub(crate) ttfb: Duration,
}

pub struct ProbeSpec<'a> {
    pub requests: &'a MediaRequestExecutor,
    pub url: &'a str,
    pub priority: PreemptionAuthority,
    pub timeouts: TransferTimeouts,
}

pub(crate) struct ObservedProbe {
    pub outcome: Result<ProbeResult>,
    pub concurrency: usize,
}

/// Probes `url` with HEAD. Range support remains unknown when the
/// header is absent; a later policy-granted body request resolves it.
pub async fn probe(spec: ProbeSpec<'_>, stats: &mut HostStats) -> Result<ProbeResult> {
    probe_observed(spec, stats).await.outcome
}

pub(crate) async fn probe_observed(spec: ProbeSpec<'_>, stats: &mut HostStats) -> ObservedProbe {
    let mut concurrency = 0;
    let outcome = describe(&spec, &mut concurrency).await;
    ObservedProbe {
        outcome: conclude(stats, spec.url, outcome),
        concurrency,
    }
}

struct ProbeFacts {
    content_length: Option<u64>,
    accept_ranges: Option<bool>,
    content_type: Option<String>,
    validator: Option<EvidenceValidator>,
    ttfb: Duration,
}

async fn describe(spec: &ProbeSpec<'_>, concurrency: &mut usize) -> Result<ProbeFacts> {
    let (head, ttfb) = send_head(spec, concurrency).await?;
    validate_response_headers(head.headers())?;
    let head = head.error_for_status().context("HEAD probe rejected")?;
    require_identity_encoding(head.headers()).context("HEAD response is encoded")?;
    origin_content_type::require_admissible(head.headers())?;
    Ok(facts_from_head(&head, ttfb))
}

async fn send_head(
    spec: &ProbeSpec<'_>,
    concurrency: &mut usize,
) -> Result<(MediaResponse, Duration)> {
    let admitted = spec
        .requests
        .get(spec.url, spec.priority)?
        .header(ACCEPT_ENCODING, HeaderValue::from_static("identity"))
        .head()
        .admit_for(spec.timeouts.admission)
        .await?;
    *concurrency = RequestAuthority::from_url(spec.url)
        .map(|authority| spec.requests.active_for(&authority))
        .unwrap_or(1)
        .max(1);
    let started = Instant::now();
    let deadline = started + spec.timeouts.headers;
    let response = await_headers(admitted.send_with_redirect_deadline(deadline), deadline).await?;
    let ttfb = response.origin_elapsed(started.elapsed());
    Ok((response, ttfb))
}

async fn await_headers(
    sending: impl Future<Output = Result<MediaResponse>>,
    deadline: Instant,
) -> Result<MediaResponse> {
    tokio::time::timeout_at(deadline, sending)
        .await
        .context("probe response headers timed out")?
        .context("probe request failed")
}

fn facts_from_head(response: &MediaResponse, ttfb: Duration) -> ProbeFacts {
    ProbeFacts {
        content_length: header_u64(response, &CONTENT_LENGTH),
        accept_ranges: accepts_byte_ranges(response),
        content_type: header_text(response, &CONTENT_TYPE),
        validator: response_validator(response),
        ttfb,
    }
}

fn conclude(stats: &mut HostStats, url: &str, outcome: Result<ProbeFacts>) -> Result<ProbeResult> {
    let host = host_of(url);
    match outcome {
        Ok(facts) => {
            if let Some(host) = &host {
                stats.record_ttfb(host, facts.ttfb.as_millis() as u64);
                stats.record_success(host);
            }
            Ok(result_from(facts))
        }
        Err(error) => {
            if let Some(host) = &host.filter(|_| !is_admission_timeout(&error)) {
                stats.record_failure(host);
            }
            Err(error)
        }
    }
}

fn result_from(facts: ProbeFacts) -> ProbeResult {
    ProbeResult {
        content_length: facts.content_length,
        accept_ranges: facts.accept_ranges,
        content_type: facts.content_type,
        validator: facts.validator,
        ttfb: facts.ttfb,
    }
}

fn header_text(response: &MediaResponse, name: &HeaderName) -> Option<String> {
    let value = response.headers().get(name)?;
    value.to_str().ok().map(str::to_owned)
}

fn header_u64(response: &MediaResponse, name: &HeaderName) -> Option<u64> {
    header_text(response, name)?.trim().parse().ok()
}

fn accepts_byte_ranges(response: &MediaResponse) -> Option<bool> {
    header_text(response, &ACCEPT_RANGES).map(|value| value.trim().eq_ignore_ascii_case("bytes"))
}

fn response_validator(response: &MediaResponse) -> Option<EvidenceValidator> {
    header_text(response, &ETAG)
        .and_then(EvidenceValidator::strong_etag)
        .or_else(|| {
            header_text(response, &LAST_MODIFIED).and_then(EvidenceValidator::last_modified)
        })
}

fn is_admission_timeout(error: &anyhow::Error) -> bool {
    error.is::<ghostr_net::media_request_executor::MediaRequestAdmissionTimeout>()
}
