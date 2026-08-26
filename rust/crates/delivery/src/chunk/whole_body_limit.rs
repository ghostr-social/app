use core::fmt::{Display, Formatter};
use ghostr_engine::adaptive::WholeBodyContract;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WholeBodyLimitReached {
    maximum_bytes: u64,
    received_bytes: u64,
}

impl WholeBodyLimitReached {
    pub(crate) fn check(
        written: u64,
        received: u64,
        contract: WholeBodyContract,
    ) -> anyhow::Result<()> {
        let observed = written.saturating_add(received);
        if observed <= contract.maximum_bytes() {
            return Ok(());
        }
        match contract {
            WholeBodyContract::Capped { maximum_bytes } => Err(Self {
                maximum_bytes,
                received_bytes: observed,
            }
            .into()),
            WholeBodyContract::Exact { .. } => {
                anyhow::bail!("whole response exceeds its exact length")
            }
        }
    }

    pub(crate) const fn maximum_bytes(self) -> u64 {
        self.maximum_bytes
    }

    pub(crate) const fn received_bytes(self) -> u64 {
        self.received_bytes
    }
}

impl Display for WholeBodyLimitReached {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "whole response exceeded its {} byte policy limit after {} bytes",
            self.maximum_bytes, self.received_bytes
        )
    }
}

impl core::error::Error for WholeBodyLimitReached {}

pub(crate) fn from_error(error: &anyhow::Error) -> Option<WholeBodyLimitReached> {
    error.downcast_ref::<WholeBodyLimitReached>().copied()
}
