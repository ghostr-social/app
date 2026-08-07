//! Composition surface over the extracted media crates. The delivery pipeline
//! now lives in `ghostr-net`, `ghostr-media-model`, `ghostr-media-store`,
//! `ghostr-hls-manifest`, `ghostr-delivery`, and `ghostr-gateway`; re-exporting
//! them here keeps `crate::video::…` as the single path the FFI layer, the
//! generated bindings, and the integration tests address the engine through.

pub use ghostr_delivery::*;
pub use ghostr_gateway::*;
pub use ghostr_hls_manifest::*;
pub use ghostr_media_model::*;
pub use ghostr_media_store::*;
pub use ghostr_net::*;
pub use ghostr_partial_store::*;

pub mod ffi_models;
pub mod native_gateway;

pub use native_gateway as video;
