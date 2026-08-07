//! Shared builders for the outbox-routing tests: a directory populated
//! from fixture kind-10002 events, and a plan executor that records what
//! it was asked to run instead of touching relays.

use crate::outbox::directory::OutboxDirectory;
use crate::plan_executor::{PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::outbox::directory::SharedOutboxDirectory;
use crate::retrieval_types::PlanFailure;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, PublicKey, Tag, Timestamp};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub const BOOTSTRAP_RELAY: &str = "wss://boot.example";

/// A kind-10002 relay list declaring one write relay.
pub fn relay_list_event(keys: &Keys, url: &str) -> Event {
    EventBuilder::new(Kind::RelayList, "")
        .tags([Tag::parse(vec!["r".to_owned(), url.to_owned()]).expect("fixture tag")])
        .custom_created_at(Timestamp::from(10))
        .sign_with_keys(keys)
        .expect("fixture event")
}

/// A kind-3 follow list naming every given pubkey.
pub fn contact_list_event(keys: &Keys, follows: &[PublicKey]) -> Event {
    let tags = follows
        .iter()
        .map(|follow| Tag::parse(vec!["p".to_owned(), follow.to_hex()]).expect("fixture tag"));
    EventBuilder::new(Kind::ContactList, "")
        .tags(tags)
        .custom_created_at(Timestamp::from(20))
        .sign_with_keys(keys)
        .expect("fixture event")
}

/// A directory whose viewer follows `count` creators, each with one
/// distinct declared write relay.
pub fn directory_with_follows(count: usize) -> OutboxDirectory {
    let mut directory = OutboxDirectory::new(vec![BOOTSTRAP_RELAY.to_owned()]);
    let follows: Vec<Keys> = (0..count).map(|_| Keys::generate()).collect();
    for (index, keys) in follows.iter().enumerate() {
        directory.ingest(&relay_list_event(
            keys,
            &format!("wss://write{index:02}.example"),
        ));
    }
    directory.track_viewer_follows(follows.iter().map(Keys::public_key).collect());
    directory
}

pub fn shared_directory(directory: OutboxDirectory) -> SharedOutboxDirectory {
    Arc::new(RwLock::new(directory))
}

/// A shared directory that knows the bootstrap relay and nothing else.
pub fn empty_directory() -> SharedOutboxDirectory {
    shared_directory(OutboxDirectory::new(vec![BOOTSTRAP_RELAY.to_owned()]))
}

/// Executor that reports every retrieval and never completes one, so a
/// test can prove nothing waits on it.
pub struct RecordingExecutor {
    pub started: mpsc::UnboundedSender<PlannedRetrieval>,
}

impl PlanExecutor for RecordingExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        let _ = self.started.send(retrieval);
        Box::pin(std::future::pending())
    }
}

pub fn recording_executor() -> (
    Arc<dyn PlanExecutor>,
    mpsc::UnboundedReceiver<PlannedRetrieval>,
) {
    let (started, retrievals) = mpsc::unbounded_channel();
    (Arc::new(RecordingExecutor { started }), retrievals)
}

/// Executor that reports every retrieval and fails it, standing in for
/// relays that were unreachable when the app started.
pub struct FailingExecutor {
    pub started: mpsc::UnboundedSender<PlannedRetrieval>,
}

impl PlanExecutor for FailingExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        let _ = self.started.send(retrieval);
        Box::pin(async { Err(PlanFailure::new("relays unreachable")) })
    }
}

pub fn failing_executor() -> (
    Arc<dyn PlanExecutor>,
    mpsc::UnboundedReceiver<PlannedRetrieval>,
) {
    let (started, retrievals) = mpsc::unbounded_channel();
    (Arc::new(FailingExecutor { started }), retrievals)
}
