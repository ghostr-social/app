//! Executes one retrieval plan against the relay pool.
//!
//! `relay_executor` implements `crate::plan_executor::PlanExecutor`, with
//! the fetch, route, and collection steps beside it. The trait stays at
//! crate root because `scheduler` and `outbox` also depend on it.

pub(crate) mod cache_fallback;
pub(crate) mod collector;
pub(crate) mod fetch;
pub mod relay_executor;
pub(crate) mod routes;
