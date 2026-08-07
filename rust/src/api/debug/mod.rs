//! The Nostr half of the loopback debug web app. Desktop debug builds
//! only — gated once at its declaration in `api::mod`, so nothing
//! inside repeats the target and feature checks.

pub mod nostr;
pub(crate) mod relay_status;
