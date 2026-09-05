//! `ghostr-net` — extracted from `rust_lib_ghostr::video`.

pub mod content_range;
pub mod identity_encoding;
pub mod internet_allowance;
pub mod media_log_identity;
pub mod media_request_executor;
pub mod media_retention;
pub mod native_cache_failure;
pub mod origin_content_type;
pub mod outbound_media_client;
mod public_dns_resolver;
mod public_media_address;
pub mod response_limits;
pub mod strong_etag;
pub mod transfer_timeouts;

#[cfg(test)]
mod tests;
