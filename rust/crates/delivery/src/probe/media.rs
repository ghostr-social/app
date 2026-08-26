//! HEAD probe service (plan Phase 1 step 5): learns a video's size,
//! range support, and content type through the SSRF-safe outbound
//! client. Body requests remain exclusively owned by policy grants.

use anyhow::{Context as _, Result};
use core::future::Future;
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::RequestAuthority;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};
use ghostr_net::origin_content_type;
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use reqwest::header::{HeaderValue, ACCEPT_ENCODING};
use tokio::time::Instant;

mod response_headers;

/// What one probe learned about a media URL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeResult {
    pub final_url: String,
    pub observed: ghostr_engine::evidence::EvidenceTime,
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
    pub network: Option<&'a dyn ProbeNetwork>,
}

pub trait ProbeNetwork: Sync {
    fn network_class(&self) -> ghostr_engine::origin_model::NetworkClass;
}

pub struct ObservedProbe {
    pub outcome: Result<ProbeResult>,
    pub(crate) concurrency: usize,
    pub(crate) network_class: ghostr_engine::origin_model::NetworkClass,
}

/// Probes `url` with HEAD. Range support remains unknown when the
/// header is absent; a later policy-granted body request resolves it.
///
/// # Errors
///
/// Returns an error when admission, transport, or response validation fails.
pub async fn probe(spec: ProbeSpec<'_>, stats: &mut HostStats) -> ObservedProbe {
    let mut concurrency = 0;
    let mut network_class = ghostr_engine::origin_model::NetworkClass::Unavailable;
    let outcome = describe(&spec, &mut concurrency, &mut network_class).await;
    ObservedProbe {
        outcome: conclude(stats, spec.url, outcome),
        concurrency,
        network_class,
    }
}

struct ProbeFacts {
    final_url: String,
    observed: ghostr_engine::evidence::EvidenceTime,
    content_length: Option<u64>,
    accept_ranges: Option<bool>,
    content_type: Option<String>,
    validator: Option<EvidenceValidator>,
    ttfb: Duration,
}

async fn describe(
    spec: &ProbeSpec<'_>,
    concurrency: &mut usize,
    network_class: &mut ghostr_engine::origin_model::NetworkClass,
) -> Result<ProbeFacts> {
    let (head, ttfb, observed) = send_head(spec, concurrency, network_class).await?;
    validate_response_headers(head.headers())?;
    let head = head.error_for_status().context("HEAD probe rejected")?;
    require_identity_encoding(head.headers()).context("HEAD response is encoded")?;
    origin_content_type::require_admissible(head.headers())?;
    Ok(facts_from_head(&head, ttfb, observed))
}

async fn send_head(
    spec: &ProbeSpec<'_>,
    concurrency: &mut usize,
    network_class: &mut ghostr_engine::origin_model::NetworkClass,
) -> Result<(
    MediaResponse,
    Duration,
    ghostr_engine::evidence::EvidenceTime,
)> {
    let admitted = spec
        .requests
        .get(spec.url, spec.priority)?
        .header(ACCEPT_ENCODING, HeaderValue::from_static("identity"))
        .head()
        .admit_for(spec.timeouts.admission)
        .await?;
    *network_class = spec
        .network
        .map_or(*network_class, ProbeNetwork::network_class);
    *concurrency = RequestAuthority::from_url(spec.url)
        .map(|authority| spec.requests.active_for(&authority))
        .unwrap_or(1)
        .max(1);
    let started = Instant::now();
    let deadline = started + spec.timeouts.headers;
    let response = await_headers(admitted.send_with_redirect_deadline(deadline), deadline).await?;
    let observed = crate::manager::time::evidence_time();
    let ttfb = response.origin_elapsed(started.elapsed());
    Ok((response, ttfb, observed))
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

fn facts_from_head(
    response: &MediaResponse,
    ttfb: Duration,
    observed: ghostr_engine::evidence::EvidenceTime,
) -> ProbeFacts {
    ProbeFacts {
        final_url: response.url().to_string(),
        observed,
        content_length: response_headers::content_length(response),
        accept_ranges: response_headers::accepts_byte_ranges(response),
        content_type: response_headers::content_type(response),
        validator: response_headers::validator(response),
        ttfb,
    }
}

fn conclude(stats: &mut HostStats, url: &str, outcome: Result<ProbeFacts>) -> Result<ProbeResult> {
    match outcome {
        Ok(facts) => {
            let host = host_of(&facts.final_url);
            if let Some(host) = &host {
                stats.record_ttfb(host, facts.ttfb.as_millis() as u64);
                stats.record_success(host);
            }
            Ok(result_from(facts))
        }
        Err(error) => {
            let host = host_of(url);
            if let Some(host) = &host.filter(|_| !is_admission_timeout(&error)) {
                stats.record_failure(host);
            }
            Err(error)
        }
    }
}

fn result_from(facts: ProbeFacts) -> ProbeResult {
    ProbeResult {
        final_url: facts.final_url,
        observed: facts.observed,
        content_length: facts.content_length,
        accept_ranges: facts.accept_ranges,
        content_type: facts.content_type,
        validator: facts.validator,
        ttfb: facts.ttfb,
    }
}

fn is_admission_timeout(error: &anyhow::Error) -> bool {
    error.is::<ghostr_net::media_request_executor::MediaRequestAdmissionTimeout>()
}
