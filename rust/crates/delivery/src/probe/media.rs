//! HEAD probe service (plan Phase 1 step 5): learns a video's size,
//! range support, and content type through the SSRF-safe outbound
//! client. Body requests remain exclusively owned by policy grants.

use anyhow::{Context as _, Result};
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::origin_model::{NetworkClass, OriginAttemptContext, OriginAttemptProfile};
use ghostr_engine::RequestAuthority;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};
use ghostr_net::origin_content_type;
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use reqwest::header::{HeaderValue, ACCEPT_ENCODING};
use tokio::time::Instant;

mod deadline;
mod response_headers;
pub(crate) use deadline::is_usefulness_timeout;

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
    pub profile: OriginAttemptProfile,
}

pub trait ProbeNetwork: Sync {
    fn network_class(&self) -> ghostr_engine::origin_model::NetworkClass;
}

pub struct ObservedProbe {
    pub outcome: Result<ProbeResult>,
    pub(crate) attempt_context: Option<OriginAttemptContext>,
}

/// Probes `url` with HEAD. Range support remains unknown when the
/// header is absent; a later policy-granted body request resolves it.
///
/// # Errors
///
/// Returns an error when admission, transport, or response validation fails.
pub async fn probe(spec: ProbeSpec<'_>, stats: &mut HostStats) -> ObservedProbe {
    let mut attempt_context = None;
    let outcome = describe(&spec, &mut attempt_context).await;
    let outcome = if attempt_context.is_some() {
        conclude(stats, spec.url, outcome)
    } else {
        outcome.map(result_from)
    };
    ObservedProbe {
        outcome,
        attempt_context,
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
    attempt_context: &mut Option<OriginAttemptContext>,
) -> Result<ProbeFacts> {
    let (head, ttfb, observed) = send_head(spec, attempt_context).await?;
    validate_response_headers(head.headers())?;
    let head = head.error_for_status().context("HEAD probe rejected")?;
    require_identity_encoding(head.headers()).context("HEAD response is encoded")?;
    origin_content_type::require_admissible(head.headers())?;
    Ok(facts_from_head(&head, ttfb, observed))
}

async fn send_head(
    spec: &ProbeSpec<'_>,
    attempt_context: &mut Option<OriginAttemptContext>,
) -> Result<(
    MediaResponse,
    Duration,
    ghostr_engine::evidence::EvidenceTime,
)> {
    let action_deadline = deadline::head_usefulness();
    let request = spec
        .requests
        .get(spec.url, spec.priority)?
        .header(ACCEPT_ENCODING, HeaderValue::from_static("identity"))
        .head();
    let admitted = deadline::admit(request, action_deadline, spec.timeouts.admission).await?;
    let network = spec
        .network
        .map_or(NetworkClass::Unavailable, ProbeNetwork::network_class);
    let concurrency = RequestAuthority::from_url(spec.url)
        .map(|authority| spec.requests.active_for(&authority))
        .unwrap_or(1)
        .max(1);
    *attempt_context = Some(OriginAttemptContext::new(
        spec.profile,
        network,
        concurrency,
        crate::manager::time::unix_time_ms(),
    ));
    let started = Instant::now();
    let response = deadline::send(admitted, action_deadline, spec.timeouts.headers).await?;
    let observed = crate::manager::time::evidence_time();
    let ttfb = response.origin_elapsed(started.elapsed());
    Ok((response, ttfb, observed))
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
            if let Some(host) = &host.filter(|_| !is_local_timeout(&error)) {
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

fn is_local_timeout(error: &anyhow::Error) -> bool {
    error.is::<ghostr_net::media_request_executor::MediaRequestAdmissionTimeout>()
        || is_usefulness_timeout(error)
}
