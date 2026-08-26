//! Ghostr's native media engine and Flutter-facing application boundary.

pub mod api;
pub use ghostr_discovery as discovery;
pub use ghostr_engine as engine;
#[allow(
    unsafe_code,
    clippy::all,
    clippy::nursery,
    clippy::pedantic,
    clippy::restriction,
    reason = "generated bridge output is excluded by policy.toml"
)]
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
pub mod video;
