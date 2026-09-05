use super::RequestLease;
use anyhow::{Context as _, Result};

impl RequestLease {
    pub(in crate::media_request_executor) fn reserve_body(&mut self, maximum: u64) -> Result<()> {
        self.body = Some(
            self.gate
                .inner
                .allowance
                .reserve(maximum)
                .context(crate::internet_allowance::InternetAdmissionDenied)?,
        );
        Ok(())
    }

    pub(in crate::media_request_executor) fn sending(&mut self) {
        if let Some(body) = &mut self.body {
            body.started();
        }
    }

    pub(in crate::media_request_executor) fn received_body(&mut self, bytes: u64) -> Result<()> {
        self.body
            .as_mut()
            .context("media request has no body reservation")?
            .received(bytes)
    }

    pub(in crate::media_request_executor) fn complete_body(&mut self) -> Result<()> {
        self.body
            .as_mut()
            .context("media request has no body reservation")?
            .complete()
    }
}
