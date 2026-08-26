use super::*;

pub(crate) async fn file_lists(sinks: &OutcomeSinks, events: &[Event]) {
    let session = lock(&sinks.state).session_generation();
    file_lists_for(sinks, session, events).await;
}
