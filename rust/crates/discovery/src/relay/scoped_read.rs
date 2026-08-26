//! Parallel relay fan-out built from exact-subscription queries.

use super::io::{RelayReadIo, RelayReadResult};
use super::scoped_query::event_limit;
use super::scoped_state::EventSink;
use crate::relay::roles::MAX_RELAY_READ_FANOUT;
use anyhow::{bail, Context as _};
use core::time::Duration;
use nostr_sdk::{Client, Filter};
use std::sync::Arc;

mod batch;
use batch::{QueryBatch, QuerySummary, QueryTemplate};

pub(super) struct ScopedReadResult {
    pub(super) result: anyhow::Result<RelayReadResult>,
    pub(super) completed_relays: Vec<String>,
    pub(super) failed_relays: Vec<String>,
}

pub(super) async fn read(
    client: Arc<Client>,
    request: RelayReadIo,
    readiness_timeout: Duration,
) -> anyhow::Result<ScopedReadResult> {
    let (sink, summary) = execute(client, request, readiness_timeout).await?;
    let mut events = sink.snapshot();
    order_union(&mut events);
    let result = relay_result(events, &sink, &summary);
    Ok(ScopedReadResult {
        result,
        completed_relays: summary.completed_relays,
        failed_relays: summary.failed_relays,
    })
}

async fn execute(
    client: Arc<Client>,
    request: RelayReadIo,
    readiness_timeout: Duration,
) -> anyhow::Result<(EventSink, QuerySummary)> {
    let RelayReadIo {
        relays,
        filter,
        timeout,
        progress,
        admissions: _,
    } = request;
    let relay_count = relays.len();
    let union_limit = read_union_limit(relay_count, &filter)?;
    let sink = EventSink::new(progress, union_limit);
    let template = QueryTemplate::new(filter, timeout, readiness_timeout, sink.clone());
    let summary = QueryBatch::start(client, relays, template)
        .await
        .finish()
        .await;
    Ok((sink, summary))
}

fn relay_result(
    events: Vec<nostr_sdk::Event>,
    sink: &EventSink,
    summary: &QuerySummary,
) -> anyhow::Result<RelayReadResult> {
    if summary.authoritative && !sink.overflowed() {
        Ok(RelayReadResult::complete(events))
    } else if summary.responded || !events.is_empty() {
        Ok(RelayReadResult {
            events,
            complete: false,
        })
    } else {
        Err(anyhow::anyhow!(failure_message(&summary.failures)))
    }
}

fn order_union(events: &mut [nostr_sdk::Event]) {
    events.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn read_union_limit(relay_count: usize, filter: &Filter) -> anyhow::Result<usize> {
    if relay_count > MAX_RELAY_READ_FANOUT {
        bail!("relay fanout exceeds {MAX_RELAY_READ_FANOUT}");
    }
    relay_count
        .checked_mul(event_limit(filter))
        .context("relay result bound overflowed")
}

fn failure_message(failures: &[String]) -> String {
    if failures.is_empty() {
        return "no relay query produced an authoritative result".to_owned();
    }
    failures.join("; ")
}
