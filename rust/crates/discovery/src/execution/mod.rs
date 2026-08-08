//! Running one retrieval plan against the relay pool: `relay_executor`
//! implements `crate::plan_executor::PlanExecutor`, with the fetch,
//! route, and collection steps beside it. The trait itself stays at the
//! crate root — `scheduler` and `outbox` depend on it too, so it must
//! not live inside one of its own implementations.

pub(crate) mod cache_fallback;
pub(crate) mod collector;
pub(crate) mod fetch;
pub mod relay_executor;
pub(crate) mod routes;
