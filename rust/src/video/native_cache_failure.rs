use anyhow::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub(crate) struct PermanentCacheFailure(&'static str);

impl Display for PermanentCacheFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for PermanentCacheFailure {}

pub fn permanent(message: &'static str) -> Error {
    permanent_cause(message).into()
}

pub(crate) fn permanent_cause(message: &'static str) -> PermanentCacheFailure {
    PermanentCacheFailure(message)
}

pub fn is_permanent(error: &Error) -> bool {
    has_permanent_cause(error)
}

fn has_permanent_cause(error: &Error) -> bool {
    error.chain().any(is_permanent_cause)
}

fn is_permanent_cause(cause: &(dyn std::error::Error + 'static)) -> bool {
    cause.downcast_ref::<PermanentCacheFailure>().is_some()
        || cause
            .downcast_ref::<io::Error>()
            .and_then(io::Error::get_ref)
            .and_then(|inner| inner.downcast_ref::<PermanentCacheFailure>())
            .is_some()
}
