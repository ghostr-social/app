//! NIP-65 outbox routing: whose relay lists we chase, where they get
//! filed, and how a filed list turns into per-author query routes.

pub mod bootstrap;
pub mod directory;
pub mod plans;
pub mod relay_list;
