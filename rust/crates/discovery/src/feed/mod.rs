//! What a feed is and how it pages: the spec that names one, the store
//! that holds its posts, the cursor that walks backwards through them,
//! and the assembly step that turns retrieved events into entries.

pub mod assembly;
pub mod cursor;
pub mod pagination;
pub mod spec;
pub mod store;
pub mod store_cursor;
