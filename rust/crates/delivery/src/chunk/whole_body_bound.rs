use core::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WholeBodyBoundDiscovered {
    maximum_bytes: u64,
    total_bytes: u64,
}

impl WholeBodyBoundDiscovered {
    pub(super) const fn new(maximum_bytes: u64, total_bytes: u64) -> Self {
        Self {
            maximum_bytes,
            total_bytes,
        }
    }
}

impl Display for WholeBodyBoundDiscovered {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "whole-body cap {} discovered representation length {}",
            self.maximum_bytes, self.total_bytes
        )
    }
}

impl core::error::Error for WholeBodyBoundDiscovered {}

pub(crate) fn from_error(error: &anyhow::Error) -> Option<WholeBodyBoundDiscovered> {
    error.downcast_ref::<WholeBodyBoundDiscovered>().copied()
}

#[cfg(test)]
#[path = "whole_body_bound_axiom_test.rs"]
pub(crate) mod axiom_test_support;
