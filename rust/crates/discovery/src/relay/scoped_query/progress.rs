//! Identification of local progress-channel backpressure.

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub(super) struct LocalBackpressure;

pub(in crate::relay) fn is_local_progress_backpressure(error: &anyhow::Error) -> bool {
    error.downcast_ref::<LocalBackpressure>().is_some()
}

impl Display for LocalBackpressure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("local progress delivery reached the query deadline")
    }
}

impl std::error::Error for LocalBackpressure {}
