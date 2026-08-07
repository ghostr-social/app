//! The relay pool: which relays are connected, in what role, and the
//! transitions between those roles. Everything here is about relay
//! identity and membership — issuing queries is `execution`'s job.

pub mod io;
pub mod pool;
pub mod registration;
pub mod removal;
pub mod role_book;
pub mod roles;
pub mod route;
pub mod transition;
pub mod url;
