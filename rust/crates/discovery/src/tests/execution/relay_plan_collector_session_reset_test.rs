use crate::execution::collector::collect_page;
use crate::execution::fetch::FetchedEvents;
use crate::query::search::QueryRole;
use crate::retrieval_types::PlanFailure;
use crate::session_generation::SESSION_RESET_MESSAGE;

#[tokio::test]
async fn session_reset_remains_fatal_when_a_sibling_succeeds() {
    let success = tokio::spawn(async { Ok(FetchedEvents::fresh(Vec::new())) });
    let reset = tokio::spawn(async { Err(PlanFailure::new(SESSION_RESET_MESSAGE)) });

    let result = collect_page(vec![
        (QueryRole::Primary, success),
        (QueryRole::Additive, reset),
    ])
    .await;
    let Err(failure) = result else {
        panic!("session generations never mix");
    };

    assert_eq!(failure.message, SESSION_RESET_MESSAGE);
}
