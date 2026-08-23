//! Lifecycle and per-relay outcomes for one exact-subscription fan-out.

use super::super::scoped_query::{
    is_local_progress_backpressure, PreparedQuery, QueryCompletion, QueryRequest, ScopedQuery,
};
use super::super::scoped_state::{CloseGuard, EventSink};
use anyhow::Context;
use nostr_sdk::{Client, Filter};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::{JoinError, JoinSet};

#[derive(Clone)]
pub(super) struct QueryTemplate {
    filter: Filter,
    timeout: Duration,
    readiness_timeout: Duration,
    sink: EventSink,
}

pub(super) struct QuerySummary {
    pub(super) authoritative: bool,
    pub(super) responded: bool,
    pub(super) failures: Vec<String>,
    pub(super) completed_relays: Vec<String>,
    pub(super) failed_relays: Vec<String>,
}

pub(super) struct QueryBatch {
    tasks: JoinSet<(String, anyhow::Result<QueryCompletion>)>,
    cancellation_guards: Vec<CloseGuard>,
    failures: Vec<String>,
    running: HashSet<String>,
    completed_relays: Vec<String>,
    failed_relays: Vec<String>,
}

impl QueryTemplate {
    pub(super) fn new(
        filter: Filter,
        timeout: Duration,
        readiness_timeout: Duration,
        sink: EventSink,
    ) -> Self {
        Self {
            filter,
            timeout,
            readiness_timeout,
            sink,
        }
    }
}

impl QueryBatch {
    pub(super) async fn start(
        client: Arc<Client>,
        relays: Vec<String>,
        template: QueryTemplate,
    ) -> Self {
        let mut batch = Self::default();
        for url in relays {
            batch.add(client.clone(), url, template.clone()).await;
        }
        batch
    }

    async fn add(&mut self, client: Arc<Client>, url: String, template: QueryTemplate) {
        let relay = match client.relay(&url).await {
            Ok(relay) => relay,
            Err(error) => {
                self.failures.push(format!("relay {url}: {error}"));
                self.failed_relays.push(url);
                return;
            }
        };
        let prepared = PreparedQuery::new(QueryRequest {
            relay,
            filter: template.filter,
            timeout: template.timeout,
            readiness_timeout: template.readiness_timeout,
            sink: template.sink,
        });
        self.cancellation_guards.push(prepared.cancellation_guard);
        self.spawn(url, prepared.query);
    }

    fn spawn(&mut self, url: String, query: ScopedQuery) {
        self.running.insert(url.clone());
        self.tasks.spawn(async move {
            let result = query
                .run()
                .await
                .with_context(|| format!("relay {url} query failed"));
            (url, result)
        });
    }

    pub(super) async fn finish(mut self) -> QuerySummary {
        let mut authoritative = self.failures.is_empty() && !self.cancellation_guards.is_empty();
        let mut responded = false;
        while let Some(joined) = self.tasks.join_next().await {
            let (completed, answered) = self.record(joined);
            authoritative &= completed;
            responded |= answered;
        }
        self.failed_relays.extend(self.running.drain());
        self.summary(authoritative, responded)
    }

    fn record(
        &mut self,
        joined: Result<(String, anyhow::Result<QueryCompletion>), JoinError>,
    ) -> (bool, bool) {
        match joined {
            Ok((url, result)) => self.record_query(url, result),
            Err(error) => {
                self.failures
                    .push(format!("relay query task failed: {error}"));
                (false, false)
            }
        }
    }

    fn record_query(
        &mut self,
        url: String,
        result: anyhow::Result<QueryCompletion>,
    ) -> (bool, bool) {
        self.running.remove(&url);
        match result {
            Ok(completion) if completion.authoritative => {
                self.completed_relays.push(url);
                (true, true)
            }
            Ok(_) => (false, true),
            Err(error) => {
                self.failures.push(format!("{error:#}"));
                if !is_local_progress_backpressure(&error) {
                    self.failed_relays.push(url);
                }
                (false, false)
            }
        }
    }

    fn summary(self, authoritative: bool, responded: bool) -> QuerySummary {
        QuerySummary {
            authoritative,
            responded,
            failures: self.failures,
            completed_relays: self.completed_relays,
            failed_relays: self.failed_relays,
        }
    }
}

impl Default for QueryBatch {
    fn default() -> Self {
        Self {
            tasks: JoinSet::new(),
            cancellation_guards: Vec::new(),
            failures: Vec::new(),
            running: HashSet::new(),
            completed_relays: Vec::new(),
            failed_relays: Vec::new(),
        }
    }
}
