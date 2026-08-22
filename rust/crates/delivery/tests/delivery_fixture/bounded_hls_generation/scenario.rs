use super::responses::{changed_full, partial, partial_with_etag, unsatisfied};
use super::{Requests, CHANGED_TOTAL_BYTES, INIT_BYTES, SHORT_INIT_BYTES};
use axum::body::Body;
use axum::http::{header, HeaderMap, Response};

#[derive(Clone, Copy)]
pub(super) enum Change {
    Once,
    Every,
    Shorter,
    Total,
    FullOnce,
    FullEvery,
}

#[derive(Clone)]
pub(super) struct FixtureState {
    pub(super) requests: Requests,
    pub(super) change: Change,
}

struct RequestedRange {
    start: usize,
    end: usize,
}

pub(super) fn init(headers: &HeaderMap, state: &FixtureState) -> Response<Body> {
    let range = headers.get(header::RANGE).unwrap().to_str().unwrap();
    let if_range = value(headers, header::IF_RANGE);
    let if_match = value(headers, header::IF_MATCH);
    let guarded = if_range.is_some();
    let requested = parse_range(range);
    let generation = observe(state, range, if_range, if_match);
    if requested.start > 0 && changed(state.change, generation) {
        return changed_response(state.change, requested, generation, guarded);
    }
    let total = match state.change {
        Change::Shorter if generation > 1 => SHORT_INIT_BYTES,
        Change::Total if generation > 1 => CHANGED_TOTAL_BYTES,
        _ => INIT_BYTES,
    };
    partial(requested.start, requested.end, generation, total)
}

fn observe(
    state: &FixtureState,
    range: &str,
    if_range: Option<String>,
    if_match: Option<String>,
) -> u8 {
    let mut seen = state.requests.lock().unwrap();
    seen.push((range.to_owned(), if_range, if_match));
    seen.iter()
        .filter(|(range, _, _)| range.starts_with("bytes=0-"))
        .count() as u8
}

fn changed(change: Change, generation: u8) -> bool {
    matches!(change, Change::Every | Change::Shorter | Change::FullEvery)
        || matches!(change, Change::Once | Change::Total | Change::FullOnce) && generation == 1
}

fn changed_response(
    change: Change,
    requested: RequestedRange,
    generation: u8,
    if_range: bool,
) -> Response<Body> {
    let next = generation.saturating_add(1);
    match change {
        Change::Shorter => unsatisfied(SHORT_INIT_BYTES, next),
        Change::Total => partial_with_etag(
            requested.start,
            requested.end,
            CHANGED_TOTAL_BYTES,
            "init-v1",
        ),
        Change::FullOnce | Change::FullEvery => changed_full(next),
        _ if if_range => changed_full(next),
        _ => partial(requested.start, requested.end, next, INIT_BYTES),
    }
}

fn parse_range(value: &str) -> RequestedRange {
    let (start, end) = value
        .strip_prefix("bytes=")
        .unwrap()
        .split_once('-')
        .unwrap();
    RequestedRange {
        start: start.parse().unwrap(),
        end: end.parse().unwrap(),
    }
}

fn value(headers: &HeaderMap, name: header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .map(|value| value.to_str().unwrap().to_owned())
}
