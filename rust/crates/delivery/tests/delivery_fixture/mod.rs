//! Harness that runs a real delivery manager against fixture servers
//! and a temp partial-range store.

#![allow(dead_code)]

pub mod aba_origin;
pub mod bounded_hls;
pub mod bounded_hls_generation;
pub mod bounded_hls_redirect;
pub mod clean_eof_origin;
pub mod concurrency_origin;
pub mod cooling_plan_origin;
pub mod decision;
pub mod demand;
mod environment;
pub mod full_disk;
pub mod gated_failure;
mod harness;
mod harness_requests;
pub mod hedge_tail_assertions;
pub mod hedge_tail_origin;
pub mod hedge_tail_stats;
pub mod hls;
pub mod hls_recovery;
pub mod host_hol;
pub mod items;
pub mod media;
pub mod options;
pub mod paced_media;
pub mod playback;
pub mod pressure_origin;
pub mod pressure_store;
pub mod probe_origins;
pub mod protected_capacity;
pub mod retry;
pub mod stats;
pub mod transient_origin;
pub mod wait;

pub use environment::{media_client, temp_directory};
pub use harness::DeliveryHarness;

pub fn start_harness(prefix: &str, options: options::DeliveryOptions) -> DeliveryHarness {
    harness::start_harness(prefix, options)
}

pub fn start_harness_at(
    root: std::path::PathBuf,
    options: options::DeliveryOptions,
) -> DeliveryHarness {
    harness::start_harness_at(root, options)
}

pub fn start_harness_with_store(
    store: std::sync::Arc<ghostr_partial_store::partial_range_store::PartialRangeStore>,
    root: std::path::PathBuf,
    options: options::DeliveryOptions,
) -> DeliveryHarness {
    harness::start_harness_with_store(store, root, options)
}

pub fn start_harness_with_requests(
    prefix: &str,
    options: options::DeliveryOptions,
    requests: ghostr_net::media_request_executor::MediaRequestExecutor,
) -> DeliveryHarness {
    harness_requests::start_harness_with_requests(prefix, options, requests)
}
