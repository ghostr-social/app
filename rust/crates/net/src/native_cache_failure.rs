use anyhow::Error;
use core::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct PermanentCacheFailure(&'static str);

impl Display for PermanentCacheFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl core::error::Error for PermanentCacheFailure {}

pub(crate) fn permanent(message: &'static str) -> Error {
    permanent_cause(message).into()
}

pub(crate) fn permanent_cause(message: &'static str) -> PermanentCacheFailure {
    PermanentCacheFailure(message)
}
