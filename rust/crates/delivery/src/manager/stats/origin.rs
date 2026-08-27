use crate::manager::transfers::ProbeObservation;
use ghostr_engine::origin_model::{ErrorReason, OriginObservation, OriginQuery};

pub(super) fn probe(done: &ProbeObservation, observed_at_ms: u64) -> Option<OriginObservation> {
    let context = done.attempt_context?.request_context();
    let query = OriginQuery::new(done.url.clone(), context);
    Some(match &done.outcome {
        Ok(result) => OriginObservation::success(query, observed_at_ms)
            .with_ttfb_ms(result.ttfb.as_millis().max(1) as u64),
        Err(error) => OriginObservation::failure(query, observed_at_ms, error_reason(error)),
    })
}

fn error_reason(error: &anyhow::Error) -> ErrorReason {
    let text = format!("{error:#}").to_ascii_lowercase();
    if text.contains("timed out") || text.contains("timeout") {
        return ErrorReason::Timeout;
    }
    if text.contains("dns") || text.contains("lookup address") {
        return ErrorReason::Dns;
    }
    if text.contains("certificate") || text.contains("tls") {
        return ErrorReason::Tls;
    }
    if text.contains("status 5") || text.contains("http 5") {
        return ErrorReason::Http5xx;
    }
    if text.contains("status 4") || text.contains("http 4") {
        return ErrorReason::Http4xx;
    }
    ErrorReason::Unknown
}
