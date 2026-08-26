//! The FFI surface over the extracted media crates.
//!
//! The delivery pipeline itself lives in `ghostr-net`, `ghostr-media-model`,
//! `ghostr-media-store`, `ghostr-partial-store`, `ghostr-hls-manifest`,
//! `ghostr-delivery`, and `ghostr-gateway`; callers name the crate that owns what
//! they need. What stays here is what the generated bindings read: the native
//! gateway and the models it hands across.

pub mod ffi_models;
pub mod native_gateway;

pub use native_gateway as video;
