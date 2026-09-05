use super::InternetAllowance;
use anyhow::{ensure, Context as _, Result};

pub(crate) struct InternetReservation {
    ledger: InternetAllowance,
    maximum: u64,
    received: u64,
    started: bool,
    settled: bool,
}

impl InternetReservation {
    pub(super) const fn new(ledger: InternetAllowance, maximum: u64) -> Self {
        Self {
            ledger,
            maximum,
            received: 0,
            started: false,
            settled: false,
        }
    }

    pub(crate) fn started(&mut self) {
        self.started = true;
    }

    /// # Errors
    /// Returns an error if a response exceeds its reserved body envelope.
    pub(crate) fn received(&mut self, bytes: u64) -> Result<()> {
        ensure!(!self.settled, "Internet reservation is already settled");
        self.received = self
            .received
            .checked_add(bytes)
            .context("Internet response overflow")?;
        ensure!(
            self.received <= self.maximum,
            "Internet response exceeded its body envelope"
        );
        Ok(())
    }

    /// Releases unused reservation only after response completion is proven.
    ///
    /// # Errors
    /// Returns an error if the settlement cannot be persisted.
    pub(crate) fn complete(&mut self) -> Result<()> {
        if self.settled {
            return Ok(());
        }
        self.settled = true;
        self.ledger.settle(self.maximum, self.received)
    }
}

impl Drop for InternetReservation {
    fn drop(&mut self) {
        if self.settled {
            return;
        }
        let charged = if self.started {
            self.maximum.max(self.received)
        } else {
            0
        };
        // Persistence failure closes admission. The last durable reservation
        // remains charged on restart, so failure cannot manufacture allowance.
        let _ = self.ledger.settle(self.maximum, charged);
    }
}
