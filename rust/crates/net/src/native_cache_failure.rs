use anyhow::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct PermanentCacheFailure(&'static str);

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
