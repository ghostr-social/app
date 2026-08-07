//! A generic batch resolves each filter's outbox lookup independently,
//! so the data-usage cap cannot spend one filter's relay budget on another.

use crate::query::events::plan_event_queries;
use crate::outbox::directory::OutboxDirectory;
use crate::execution::relay_executor::RelayPlanExecutor;
use crate::query::search::plan_discovery;
use crate::tests::outbox_support::{shared_directory, BOOTSTRAP_RELAY};
use crate::query::video_filters::DiscoveryRequest;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Client, EventBuilder, Filter, Keys, Kind, Tag, Timestamp};
use std::sync::Arc;

#[tokio::test]
async fn mixed_batch_keeps_a_full_relay_budget_for_every_filter() {
    let first = Keys::generate();
    let second = Keys::generate();
    let discovery = Keys::generate();
    let mut directory = OutboxDirectory::new(vec![BOOTSTRAP_RELAY.to_owned()]);
    add_relays(&mut directory, &first, "a", 6);
    add_relays(&mut directory, &second, "b", 6);
    add_relays(&mut directory, &discovery, "d", 2);
    directory.track_viewer_follows(vec![discovery.public_key()]);
    let executor = RelayPlanExecutor::new(
        Arc::new(Client::default()),
        Vec::new(),
        shared_directory(directory),
        DataUsageLevel::Conservative,
    );
    let plan = plan_event_queries(vec![
        Filter::new()
            .kind(Kind::Reaction)
            .author(first.public_key()),
        Filter::new()
            .kind(Kind::Comment)
            .author(second.public_key()),
        Filter::new().kind(Kind::TextNote),
    ]);

    let routed = executor.plan_outbox_relays(&plan).await;

    assert_eq!(routed[0], Some(expected("a", 6)));
    assert_eq!(routed[1], Some(expected("b", 6)));
    assert_eq!(routed[2], Some(expected("d", 2)));
}

#[tokio::test]
async fn search_plan_skips_its_shared_outbox_lookup() {
    let executor = RelayPlanExecutor::new(
        Arc::new(Client::default()),
        Vec::new(),
        shared_directory(OutboxDirectory::new(vec![BOOTSTRAP_RELAY.to_owned()])),
        DataUsageLevel::Conservative,
    );
    let plan = plan_discovery(&DiscoveryRequest {
        search_query: Some("ghost".to_owned()),
        ..DiscoveryRequest::default()
    });

    let routed = executor.plan_outbox_relays(&plan).await;

    assert!(routed.iter().all(Option::is_none));
}

fn add_relays(directory: &mut OutboxDirectory, keys: &Keys, prefix: &str, count: usize) {
    let tags = urls(prefix, count)
        .into_iter()
        .map(|url| Tag::parse(vec!["r".to_owned(), url]).expect("relay tag"));
    let event = EventBuilder::new(Kind::RelayList, "")
        .tags(tags)
        .custom_created_at(Timestamp::from(10))
        .sign_with_keys(keys)
        .expect("relay list");
    directory.ingest(&event);
}

fn expected(prefix: &str, count: usize) -> Vec<String> {
    let mut expected = vec![BOOTSTRAP_RELAY.to_owned()];
    expected.extend(urls(prefix, count));
    expected
}

fn urls(prefix: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("wss://{prefix}{index:02}.example"))
        .collect()
}
