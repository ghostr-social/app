//! Feed state on the API side, and the mapping from discovery's types
//! into the FFI shapes Dart reads. `state` is the one locked store;
//! everything else decides what goes into it or projects what comes
//! out. The FFI entry points themselves stay in `api::feed_control`.

pub(crate) mod decisions;
pub(crate) mod mapping;
pub(crate) mod outcome_pump;
pub(crate) mod outcomes;
pub(crate) mod progress;
pub(crate) mod projection;
pub(crate) mod state;
