use crate::manager::transfers::ProbeObservation;
use ghostr_engine::origin_model::{
    ErrorReason, MediaClass, NetworkClass, OriginContext, OriginObservation, OriginQuery,
    RequestMethod,
};

pub(super) fn probe(done: &ProbeObservation, observed_at_ms: u64) -> OriginObservation {
    let bytes = done
        .outcome
        .as_ref()
        .ok()
        .and_then(|result| result.content_length)
        .unwrap_or_default();
    let media = done
        .outcome
        .as_ref()
        .ok()
        .map_or(MediaClass::Unknown, |result| {
            media_class(result.content_type.as_deref())
        });
    let context = OriginContext::new(RequestMethod::Head, bytes, media)
        .with_network(NetworkClass::Unavailable)
        .with_concurrency(done.concurrency)
        .with_observed_at_ms(observed_at_ms);
    let query = OriginQuery::new(done.url.clone(), context);
    match &done.outcome {
        Ok(result) => OriginObservation::success(query, observed_at_ms)
            .with_ttfb_ms(result.ttfb.as_millis().max(1) as u64),
        Err(error) => OriginObservation::failure(query, observed_at_ms, error_reason(error)),
    }
}

fn media_class(content_type: Option<&str>) -> MediaClass {
    let content_type = content_type.unwrap_or_default().to_ascii_lowercase();
    if content_type.contains("mpegurl") || content_type.contains("dash+xml") {
        return MediaClass::Segmented;
    }
    MediaClass::Unknown
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
