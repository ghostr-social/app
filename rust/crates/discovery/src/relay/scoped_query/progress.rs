//! Identification of local progress-channel backpressure.

use core::fmt::{Display, Formatter};

#[derive(Debug)]
pub(super) struct LocalBackpressure;

pub(in crate::relay) fn is_local_progress_backpressure(error: &anyhow::Error) -> bool {
    error.downcast_ref::<LocalBackpressure>().is_some()
}

impl Display for LocalBackpressure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("local progress delivery reached the query deadline")
    }
}

impl core::error::Error for LocalBackpressure {}
