//! HEAD probe service (plan Phase 1 step 5): learns a video's size,
//! range support, and content type through the SSRF-safe outbound
//! client, falling back to a one-byte ranged GET when the server
//! rejects HEAD. Every probe outcome feeds the per-host model.

use anyhow::{Context, Result};
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_net::content_range;
use ghostr_net::origin_content_type;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use reqwest::header::{
    HeaderName, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE,
};
use reqwest::{Method, Response, StatusCode};
use std::future::Future;
use std::time::Duration;
use tokio::time::Instant;

/// What one probe learned about a media URL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeResult {
    pub content_length: Option<u64>,
    pub accept_ranges: bool,
    pub content_type: Option<String>,
    pub(crate) ttfb: Duration,
}

/// Probes `url` with a HEAD request; when the server rejects HEAD it
/// falls back to `GET Range: bytes=0-0` (206 proves range support and
/// carries the total length). Success records TTFB and a success
/// sample for the URL's host; any failure records a host failure.
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
    accept_ranges: bool,
    content_type: Option<String>,
}

async fn describe(client: &dyn MediaHttpRequests, url: &str, wait: Duration) -> Result<ProbeFacts> {
    let head = send_head(client, url, wait).await?;
    if head.status().is_success() {
        origin_content_type::require_admissible(head.headers())?;
        return Ok(facts_from_head(&head));
    }
    facts_from_ranged_get(send_ranged_get(client, url, wait).await?)
}

async fn send_head(client: &dyn MediaHttpRequests, url: &str, wait: Duration) -> Result<Response> {
    let (inner, request) = client.get(url)?.build_split();
    let mut request = request.context("build probe request")?;
    *request.method_mut() = Method::HEAD;
    await_headers(inner.execute(request), wait).await
}

async fn send_ranged_get(
    client: &dyn MediaHttpRequests,
    url: &str,
    wait: Duration,
) -> Result<Response> {
    await_headers(client.get(url)?.header(RANGE, "bytes=0-0").send(), wait).await
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
    }
}

fn facts_from_ranged_get(response: Response) -> Result<ProbeFacts> {
    if response.status() == StatusCode::PARTIAL_CONTENT {
        origin_content_type::require_admissible(response.headers())?;
        return Ok(ProbeFacts {
            content_length: partial_total(&response),
            accept_ranges: true,
            content_type: header_text(&response, &CONTENT_TYPE),
        });
    }
    let response = response
        .error_for_status()
        .context("probe fallback rejected")?;
    origin_content_type::require_admissible(response.headers())?;
    Ok(ProbeFacts {
        content_length: header_u64(&response, &CONTENT_LENGTH),
        accept_ranges: false,
        content_type: header_text(&response, &CONTENT_TYPE),
    })
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

fn accepts_byte_ranges(response: &Response) -> bool {
    header_text(response, &ACCEPT_RANGES)
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("bytes"))
}

fn partial_total(response: &Response) -> Option<u64> {
    let value = header_text(response, &CONTENT_RANGE)?;
    content_range::parse(&value)?.1
}
