use axum::http::{header, HeaderMap, Method};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OriginRequest {
    pub method: Method,
    pub range: Option<String>,
}

#[derive(Clone, Default)]
pub(super) struct RequestLedger(Arc<Mutex<Vec<OriginRequest>>>);

impl RequestLedger {
    pub(super) fn record(&self, method: Method, headers: &HeaderMap) {
        self.0.lock().expect("request ledger").push(OriginRequest {
            method,
            range: headers
                .get(header::RANGE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
        });
    }

    pub(super) fn snapshot(&self) -> Vec<OriginRequest> {
        self.0.lock().expect("request ledger").clone()
    }

    pub(super) fn get_ranges(&self) -> Vec<core::ops::Range<u64>> {
        let mut ranges: Vec<_> = self
            .snapshot()
            .into_iter()
            .filter(|request| request.method == Method::GET)
            .filter_map(|request| request.range.as_deref().and_then(parse_range))
            .collect();
        ranges.sort_by_key(|range| (range.start, range.end));
        ranges
    }
}

fn parse_range(value: &str) -> Option<core::ops::Range<u64>> {
    let (start, end) = value.strip_prefix("bytes=")?.split_once('-')?;
    Some(start.parse().ok()?..end.parse::<u64>().ok()?.saturating_add(1))
}
