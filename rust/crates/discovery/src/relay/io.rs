//! External nostr-sdk network calls behind the relay-pool owner.

use core::future::Future;
use core::pin::Pin;
use core::time::Duration;
use nostr_sdk::{Event, Filter};

use crate::relay::health::RelayAdmissionBatch;
use crate::retrieval_types::EventProgress;

mod result;
mod sdk;

pub use result::RelayReadResult;
pub(crate) use sdk::SdkRelayIo;

pub type RelayIoFuture<'a, T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'a>>;

pub struct RelayReadIo {
    pub(crate) relays: Vec<String>,
    pub(crate) filter: Filter,
    pub(crate) timeout: Duration,
    pub(crate) progress: Option<EventProgress>,
    pub(crate) admissions: Option<RelayAdmissionBatch>,
}

pub struct RelayBroadcastIo {
    pub(crate) relays: Vec<String>,
    pub(crate) event: Event,
}

pub trait RelayIo: Send + Sync {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, RelayReadResult>;
    fn broadcast(&self, request: RelayBroadcastIo) -> RelayIoFuture<'_, ()>;
}

#[cfg(test)]
#[path = "io_axiom_test.rs"]
pub(crate) mod axiom_test_support;
