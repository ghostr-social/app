use super::delivery_fixture::full_disk::{limits, spaced_store};
use super::delivery_fixture::options::{base_params, DeliveryOptions};
use super::delivery_fixture::{start_harness_with_store, DeliveryHarness};
use super::CHUNK;
use std::sync::Arc;

pub fn constrained_harness(prefix: &str, budget: u64) -> DeliveryHarness {
    let fixture = spaced_store(prefix, limits(budget, 0), budget);
    start_harness_with_store(Arc::new(fixture.store), fixture.root, options())
}

fn options() -> DeliveryOptions {
    let mut params = base_params();
    params.chunk_bytes = CHUNK;
    params.conservative_concurrency = 1;
    params.balanced_concurrency = 1;
    params.aggressive_concurrency = 1;
    DeliveryOptions {
        params,
        ..DeliveryOptions::default()
    }
}
