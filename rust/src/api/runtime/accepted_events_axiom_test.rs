use super::*;

use crate::api::feed::outcomes::axiom_test_support::file_lists;

use crate::discovery::cache::EventCache;

pub(crate) async fn remember_accepted(cache: &EventCache, sinks: &OutcomeSinks, event: &Event) {
    cache
        .remember_for(SessionGeneration::initial(), core::slice::from_ref(event))
        .await;
    file_lists(sinks, core::slice::from_ref(event)).await;
}
