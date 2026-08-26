//! One-shot callers waiting on generic scheduler work.

use crate::plan_executor::PlanPage;
use crate::retrieval_types::{FeedContext, PlanFailure};
use nostr_sdk::Event;
use std::collections::HashMap;
use tokio::sync::oneshot;

pub(crate) type QueryResult = Result<Vec<Event>, PlanFailure>;
type PageResult = Result<PlanPage, PlanFailure>;

const INCOMPLETE_QUERY_MESSAGE: &str = "relay query did not complete authoritatively";

#[derive(Default)]
pub(crate) struct QueryBook {
    pending: HashMap<FeedContext, oneshot::Sender<QueryResult>>,
}

impl QueryBook {
    pub(super) fn reset_session(&mut self) {
        self.pending.clear();
    }

    pub(super) fn register(&mut self, context: FeedContext, reply: oneshot::Sender<QueryResult>) {
        self.pending.insert(context, reply);
    }

    pub(super) fn finish(
        &mut self,
        context: &FeedContext,
        result: PageResult,
    ) -> Result<(), PageResult> {
        let Some(reply) = self.pending.remove(context) else {
            return Err(result);
        };
        let _ = reply.send(complete(result));
        Ok(())
    }
}

fn complete(result: PageResult) -> QueryResult {
    let page = result?;
    page.complete
        .then_some(page.events)
        .ok_or_else(|| PlanFailure::new(INCOMPLETE_QUERY_MESSAGE))
}
