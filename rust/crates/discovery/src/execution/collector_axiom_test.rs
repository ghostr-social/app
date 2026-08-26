use super::*;

pub(crate) async fn collect_events(
    fetches: Vec<(QueryRole, FetchHandle)>,
) -> Result<Vec<Event>, PlanFailure> {
    collect_page(fetches).await.map(|page| page.events)
}
