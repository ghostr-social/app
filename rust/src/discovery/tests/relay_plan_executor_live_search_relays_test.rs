use crate::discovery::relay_plan_executor::RelayPlanExecutor;
use crate::discovery::tests::outbox_support::empty_directory;
use crate::engine::DataUsageLevel;
use nostr_sdk::Client;
use std::sync::Arc;

#[test]
fn configured_search_relays_are_replaceable_for_existing_executors() {
    let executor = RelayPlanExecutor::new(
        Arc::new(Client::default()),
        vec!["wss://old.example".to_owned()],
        empty_directory(),
        DataUsageLevel::Balanced,
    );
    let scheduled_executor = executor.clone();

    executor.set_search_relays(vec!["wss://new.example".to_owned()]);

    assert_eq!(
        scheduled_executor.search_relays(),
        vec!["wss://new.example"]
    );
}
