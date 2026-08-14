//! Fair relay-hint selection and bounded routing for target filters.

use crate::query::events::{plan_hinted_event_queries, HintedEventFilter};
use crate::query::search::QueryPlan;
use crate::query::video_filters::WIDE_QUERY_LIMIT;
use crate::relay::url::normalize_untrusted_relay_url;
use nostr_sdk::Filter;
use std::collections::BTreeSet;

const MAX_HINTS_PER_FILTER: usize = 8;
const MAX_HINTS_PER_PAGE: usize = WIDE_QUERY_LIMIT * 2;
pub(super) const MAX_HINTS_PER_TARGET: usize = 4;

pub(super) fn normalized_hints(hints: Vec<String>) -> Vec<String> {
    hints
        .into_iter()
        .filter_map(|hint| normalize_untrusted_relay_url(&hint))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(MAX_HINTS_PER_TARGET)
        .collect()
}

pub(super) struct TargetFilter {
    filter: Filter,
    hints: Vec<String>,
}

impl TargetFilter {
    pub(super) fn new<T>(mut filter: Filter, values: &[(T, BTreeSet<String>)]) -> Self {
        filter = filter.limit(values.len());
        Self {
            filter,
            hints: selected_hints(values),
        }
    }
}

pub(super) fn build_plan(mut targets: Vec<TargetFilter>) -> Option<QueryPlan> {
    if targets.is_empty() {
        return None;
    }
    cap_hints(&mut targets);
    let filters = targets
        .into_iter()
        .map(|target| HintedEventFilter::new(target.filter, target.hints))
        .collect();
    Some(plan_hinted_event_queries(filters))
}

fn selected_hints<T>(values: &[(T, BTreeSet<String>)]) -> Vec<String> {
    let mut selected = Vec::new();
    for hints in values.iter().map(|(_, hints)| hints) {
        if let Some(hint) = hints.first() {
            push_hint(&mut selected, hint);
        }
    }
    for hint in values.iter().flat_map(|(_, hints)| hints.iter().skip(1)) {
        if selected.len() >= MAX_HINTS_PER_FILTER {
            break;
        }
        push_hint(&mut selected, hint);
    }
    selected
}

fn push_hint(selected: &mut Vec<String>, hint: &str) {
    let available = selected.len() < MAX_HINTS_PER_FILTER;
    if available && !selected.iter().any(|value| value == hint) {
        selected.push(hint.to_owned());
    }
}

fn cap_hints(targets: &mut [TargetFilter]) {
    let mut admitted = BTreeSet::new();
    for target in targets {
        target.hints.retain(|hint| {
            admitted.contains(hint)
                || (admitted.len() < MAX_HINTS_PER_PAGE && admitted.insert(hint.clone()))
        });
    }
}
