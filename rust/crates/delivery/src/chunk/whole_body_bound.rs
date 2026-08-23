use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WholeBodyBoundDiscovered {
    maximum_bytes: u64,
    total_bytes: u64,
}

impl WholeBodyBoundDiscovered {
    pub(crate) const fn new(maximum_bytes: u64, total_bytes: u64) -> Self {
        Self {
            maximum_bytes,
            total_bytes,
        }
    }

    #[cfg(test)]
    pub(crate) const fn maximum_bytes(self) -> u64 {
        self.maximum_bytes
    }

    #[cfg(test)]
    pub(crate) const fn total_bytes(self) -> u64 {
        self.total_bytes
    }
}

impl Display for WholeBodyBoundDiscovered {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "whole-body cap {} discovered representation length {}",
            self.maximum_bytes, self.total_bytes
        )
    }
}

impl std::error::Error for WholeBodyBoundDiscovered {}

pub(crate) fn from_error(error: &anyhow::Error) -> Option<WholeBodyBoundDiscovered> {
    error.downcast_ref::<WholeBodyBoundDiscovered>().copied()
}
