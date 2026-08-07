//! One-shot callers waiting on generic scheduler work.

use crate::plan_executor::PlanFailure;
use crate::retrieval_queue::FeedContext;
use nostr_sdk::Event;
use std::collections::HashMap;
use tokio::sync::oneshot;

pub(crate) type QueryResult = Result<Vec<Event>, PlanFailure>;

#[derive(Default)]
pub(crate) struct QueryBook {
    pending: HashMap<FeedContext, oneshot::Sender<QueryResult>>,
}

impl QueryBook {
    pub(crate) fn reset_session(&mut self) {
        self.pending.clear();
    }

    pub(crate) fn register(&mut self, context: FeedContext, reply: oneshot::Sender<QueryResult>) {
        self.pending.insert(context, reply);
    }

    pub(crate) fn finish(
        &mut self,
        context: &FeedContext,
        result: QueryResult,
    ) -> Result<(), QueryResult> {
        let Some(reply) = self.pending.remove(context) else {
            return Err(result);
        };
        let _ = reply.send(result);
        Ok(())
    }
}
