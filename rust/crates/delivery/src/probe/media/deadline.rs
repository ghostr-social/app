use anyhow::{Context as _, Result};
use core::fmt;
use core::time::Duration;
use ghostr_net::media_request_executor::{
    AdmittedMediaRequest, MediaRequest, MediaRequestAdmissionTimeout, MediaResponse,
};
use tokio::time::Instant;

/// Bootstrap guard for advisory HEAD; body transfers keep their longer limits.
const HEAD_USEFULNESS_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug)]
struct HeadProbeUsefulnessTimeout;

impl fmt::Display for HeadProbeUsefulnessTimeout {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HEAD probe usefulness deadline elapsed")
    }
}

impl core::error::Error for HeadProbeUsefulnessTimeout {}

struct PhaseDeadline {
    at: Instant,
    usefulness_limited: bool,
}

pub(super) fn head_usefulness() -> Instant {
    Instant::now() + HEAD_USEFULNESS_TIMEOUT
}

pub(super) async fn admit(
    request: MediaRequest,
    action: Instant,
    phase_limit: Duration,
) -> Result<AdmittedMediaRequest> {
    let deadline = phase(action, phase_limit);
    let result = request.admit_for(deadline.remaining()).await;
    match result {
        Err(error) if deadline.usefulness_limited && error.is::<MediaRequestAdmissionTimeout>() => {
            Err(HeadProbeUsefulnessTimeout.into())
        }
        result => result,
    }
}

pub(super) async fn send(
    request: AdmittedMediaRequest,
    action: Instant,
    phase_limit: Duration,
) -> Result<MediaResponse> {
    let deadline = phase(action, phase_limit);
    let sending = request.send_with_redirect_deadline(deadline.at);
    match tokio::time::timeout_at(deadline.at, sending).await {
        Ok(Err(error))
            if deadline.usefulness_limited && error.is::<MediaRequestAdmissionTimeout>() =>
        {
            Err(HeadProbeUsefulnessTimeout.into())
        }
        Ok(result) => result.context("probe request failed"),
        Err(_) if deadline.usefulness_limited => Err(HeadProbeUsefulnessTimeout.into()),
        Err(error) => Err(error).context("probe response headers timed out"),
    }
}

pub(crate) fn is_usefulness_timeout(error: &anyhow::Error) -> bool {
    error.is::<HeadProbeUsefulnessTimeout>()
}

fn phase(action: Instant, phase_limit: Duration) -> PhaseDeadline {
    let phase = Instant::now() + phase_limit;
    PhaseDeadline {
        at: action.min(phase),
        usefulness_limited: action <= phase,
    }
}

impl PhaseDeadline {
    fn remaining(&self) -> Duration {
        self.at.saturating_duration_since(Instant::now())
    }
}
