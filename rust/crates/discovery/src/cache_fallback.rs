//! Offline answer policy for one failed relay query.

use crate::event_cache::EventCache;
use crate::plan_executor::PlanFailure;
use crate::session_generation::{SessionGeneration, SESSION_RESET_MESSAGE};
use nostr_sdk::{Event, Filter};

pub(crate) async fn cached_or_failure(
    cache: &EventCache,
    session: SessionGeneration,
    filter: &Filter,
    error: impl ToString,
) -> Result<Vec<Event>, PlanFailure> {
    let Some(stored) = cache.stored_for(session, filter).await else {
        return Err(PlanFailure::new(SESSION_RESET_MESSAGE));
    };
    if stored.is_empty() {
        return Err(PlanFailure::new(error.to_string()));
    }
    Ok(stored)
}
