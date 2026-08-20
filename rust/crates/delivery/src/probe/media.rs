//! HEAD probe service (plan Phase 1 step 5): learns a video's size,
//! range support, and content type through the SSRF-safe outbound
//! client. Body requests remain exclusively owned by policy grants.

use anyhow::{Context, Result};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::origin_content_type;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use reqwest::header::{
    HeaderName, HeaderValue, ACCEPT_ENCODING, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE, ETAG,
    LAST_MODIFIED,
};
use reqwest::{Method, Response};
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

/// Probes `url` with HEAD. Range support remains unknown when the
/// header is absent; a later policy-granted body request resolves it.
pub async fn probe(
    client: &dyn MediaHttpRequests,
    url: &str,
    timeouts: TransferTimeouts,
    stats: &mut HostStats,
) -> Result<ProbeResult> {
    let started = Instant::now();
    let outcome = describe(client, url, timeouts.headers).await;
    conclude(stats, url, started.elapsed(), outcome)
}

struct ProbeFacts {
    content_length: Option<u64>,
    accept_ranges: Option<bool>,
    content_type: Option<String>,
    validator: Option<EvidenceValidator>,
}

async fn describe(client: &dyn MediaHttpRequests, url: &str, wait: Duration) -> Result<ProbeFacts> {
    let head = send_head(client, url, wait).await?;
    validate_response_headers(head.headers())?;
    let head = head.error_for_status().context("HEAD probe rejected")?;
    require_identity_encoding(head.headers()).context("HEAD response is encoded")?;
    origin_content_type::require_admissible(head.headers())?;
    Ok(facts_from_head(&head))
}

async fn send_head(client: &dyn MediaHttpRequests, url: &str, wait: Duration) -> Result<Response> {
    let (inner, request) = client.get(url)?.build_split();
    let mut request = request.context("build probe request")?;
    *request.method_mut() = Method::HEAD;
    request
        .headers_mut()
        .insert(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
    await_headers(inner.execute(request), wait).await
}

async fn await_headers(
    sending: impl Future<Output = reqwest::Result<Response>>,
    wait: Duration,
) -> Result<Response> {
    tokio::time::timeout(wait, sending)
        .await
        .context("probe response headers timed out")?
        .context("probe request failed")
}

fn facts_from_head(response: &Response) -> ProbeFacts {
    ProbeFacts {
        content_length: header_u64(response, &CONTENT_LENGTH),
        accept_ranges: accepts_byte_ranges(response),
        content_type: header_text(response, &CONTENT_TYPE),
        validator: response_validator(response),
    }
}

fn conclude(
    stats: &mut HostStats,
    url: &str,
    ttfb: Duration,
    outcome: Result<ProbeFacts>,
) -> Result<ProbeResult> {
    let host = host_of(url);
    match outcome {
        Ok(facts) => {
            if let Some(host) = &host {
                stats.record_ttfb(host, ttfb.as_millis() as u64);
                stats.record_success(host);
            }
            Ok(result_from(facts, ttfb))
        }
        Err(error) => {
            if let Some(host) = &host {
                stats.record_failure(host);
            }
            Err(error)
        }
    }
}

fn result_from(facts: ProbeFacts, ttfb: Duration) -> ProbeResult {
    ProbeResult {
        content_length: facts.content_length,
        accept_ranges: facts.accept_ranges,
        content_type: facts.content_type,
        validator: facts.validator,
        ttfb,
    }
}

fn header_text(response: &Response, name: &HeaderName) -> Option<String> {
    let value = response.headers().get(name)?;
    value.to_str().ok().map(str::to_owned)
}

fn header_u64(response: &Response, name: &HeaderName) -> Option<u64> {
    header_text(response, name)?.trim().parse().ok()
}

fn accepts_byte_ranges(response: &Response) -> Option<bool> {
    header_text(response, &ACCEPT_RANGES).map(|value| value.trim().eq_ignore_ascii_case("bytes"))
}

fn response_validator(response: &Response) -> Option<EvidenceValidator> {
    header_text(response, &ETAG)
        .and_then(EvidenceValidator::strong_etag)
        .or_else(|| {
            header_text(response, &LAST_MODIFIED).and_then(EvidenceValidator::last_modified)
        })
}
