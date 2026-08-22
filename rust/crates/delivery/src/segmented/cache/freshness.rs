use super::generation::HlsCacheMetadata;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_net::strong_etag::single_strong_etag;
use reqwest::header::{HeaderMap, HeaderName, AGE, CACHE_CONTROL, DATE, LAST_MODIFIED, VARY};
use std::time::{Duration, Instant, SystemTime};
use url::Url;

impl HlsCacheMetadata {
    #[cfg(test)]
    pub(in crate::segmented) fn from_headers(headers: &HeaderMap) -> Self {
        Self::from_parts(headers, Duration::ZERO, true)
    }

    pub(in crate::segmented) fn from_response(
        request_url: &str,
        final_url: &Url,
        headers: &HeaderMap,
        response_delay_bound: Duration,
    ) -> Self {
        let same_url = super::super::source_key::canonical(request_url)
            == super::super::source_key::canonical(final_url.as_str());
        Self::from_parts(headers, response_delay_bound, same_url)
    }

    fn from_parts(headers: &HeaderMap, response_delay: Duration, same_url: bool) -> Self {
        let validator = response_validator(headers);
        let fresh_until = validator
            .as_ref()
            .filter(|_| same_url)
            .and_then(|_| freshness(headers, response_delay))
            .and_then(|duration| Instant::now().checked_add(duration));
        Self {
            validator,
            fresh_until,
        }
    }
}

pub(super) fn response_validator(headers: &HeaderMap) -> Option<EvidenceValidator> {
    strong_etag(headers).or_else(|| last_modified(headers))
}

fn strong_etag(headers: &HeaderMap) -> Option<EvidenceValidator> {
    single_strong_etag(headers)
        .ok()
        .flatten()
        .and_then(|value| value.to_ascii().map(str::to_owned))
        .and_then(EvidenceValidator::strong_etag)
}

fn last_modified(headers: &HeaderMap) -> Option<EvidenceValidator> {
    let value = single_header(headers, &LAST_MODIFIED)??;
    httpdate::parse_http_date(value).ok()?;
    EvidenceValidator::last_modified(value)
}

fn freshness(headers: &HeaderMap, response_delay: Duration) -> Option<Duration> {
    let directives = cache_directives(headers)?;
    if directives.iter().any(|value| forbids_reuse(value)) || !supported_vary(headers) {
        return None;
    }
    let maximum = Duration::from_secs(explicit_max_age(&directives)?);
    let age = current_age(headers)?.checked_add(response_delay)?;
    let remaining = maximum.checked_sub(age)?;
    (!remaining.is_zero()).then_some(remaining)
}

fn cache_directives(headers: &HeaderMap) -> Option<Vec<String>> {
    let mut directives = Vec::new();
    for value in headers.get_all(CACHE_CONTROL) {
        directives.extend(split_directives(value.to_str().ok()?)?);
    }
    Some(directives)
}

fn split_directives(value: &str) -> Option<Vec<String>> {
    let mut directives = Vec::new();
    let (mut start, mut quoted, mut escaped) = (0, false, false);
    for (index, character) in value.char_indices() {
        match character {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            ',' if !quoted => {
                directives.push(directive(&value[start..index])?);
                start = index + 1;
            }
            _ => {}
        }
    }
    if quoted || escaped {
        return None;
    }
    directives.push(directive(&value[start..])?);
    Some(directives)
}

fn directive(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_ascii_lowercase())
}

fn forbids_reuse(directive: &str) -> bool {
    matches!(directive_name(directive), "no-cache" | "no-store")
}

fn explicit_max_age(directives: &[String]) -> Option<u64> {
    let mut values = directives
        .iter()
        .filter(|value| directive_name(value) == "max-age")
        .map(|value| directive_value(value).and_then(delta_seconds));
    let value = values.next()??;
    values.next().is_none().then_some(value)
}

fn directive_name(value: &str) -> &str {
    value.split_once('=').map_or(value, |(name, _)| name.trim())
}

fn directive_value(value: &str) -> Option<&str> {
    let (_, value) = value.split_once('=')?;
    let value = value.trim();
    match value.strip_prefix('"') {
        Some(inner) => inner.strip_suffix('"').filter(|inner| !inner.contains('"')),
        None => (!value.contains('"')).then_some(value),
    }
}

fn delta_seconds(value: &str) -> Option<u64> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())?
}

fn current_age(headers: &HeaderMap) -> Option<Duration> {
    let age = match single_header(headers, &AGE)? {
        Some(value) => Duration::from_secs(value.trim().parse().ok()?),
        None => Duration::ZERO,
    };
    Some(age.max(apparent_age(headers)?))
}

fn apparent_age(headers: &HeaderMap) -> Option<Duration> {
    let Some(value) = single_header(headers, &DATE)? else {
        return Some(Duration::ZERO);
    };
    let date = httpdate::parse_http_date(value).ok()?;
    Some(SystemTime::now().duration_since(date).unwrap_or_default())
}

fn supported_vary(headers: &HeaderMap) -> bool {
    headers.get_all(VARY).iter().all(|value| {
        value.to_str().is_ok_and(|value| {
            value
                .split(',')
                .all(|name| name.trim().eq_ignore_ascii_case("accept-encoding"))
        })
    })
}

fn single_header<'a>(headers: &'a HeaderMap, name: &HeaderName) -> Option<Option<&'a str>> {
    let mut values = headers.get_all(name).iter();
    let first = values.next();
    if values.next().is_some() {
        return None;
    }
    match first {
        Some(value) => value.to_str().ok().map(Some),
        None => Some(None),
    }
}
