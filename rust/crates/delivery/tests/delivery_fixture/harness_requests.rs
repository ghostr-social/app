use super::harness::{start_harness_config, DeliveryHarness};
use super::options::DeliveryOptions;
use super::temp_directory;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn start_harness_with_requests(
    prefix: &str,
    options: DeliveryOptions,
    requests: ghostr_net::media_request_executor::MediaRequestExecutor,
) -> DeliveryHarness {
    let root = temp_directory(prefix);
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    start_harness_config(Arc::new(store), root, options, requests)
}
