use super::{QueryCompletion, ScopedQuery};
use anyhow::bail;
use nostr_sdk::message::MachineReadablePrefix;

impl ScopedQuery {
    pub(super) fn handle_authenticated(&mut self) -> anyhow::Result<Option<QueryCompletion>> {
        self.authenticated = true;
        self.authentication_failed = false;
        if self.awaiting_auth {
            self.awaiting_auth = false;
            self.retry_request()?;
        }
        Ok(None)
    }

    pub(super) fn handle_authentication_failed(
        &mut self,
    ) -> anyhow::Result<Option<QueryCompletion>> {
        self.authenticated = false;
        self.authentication_failed = true;
        if self.awaiting_auth {
            bail!("relay authentication failed");
        }
        Ok(None)
    }

    pub(super) fn handle_closed(
        &mut self,
        message: String,
    ) -> anyhow::Result<Option<QueryCompletion>> {
        if !is_auth_required(&message) || self.auth_retried {
            bail!("relay closed query: {message}");
        }
        self.auth_retried = true;
        if self.authentication_failed {
            bail!("relay authentication failed");
        }
        if self.authenticated {
            self.retry_request()?;
        } else {
            self.awaiting_auth = true;
        }
        Ok(None)
    }
}

fn is_auth_required(message: &str) -> bool {
    MachineReadablePrefix::parse(message) == Some(MachineReadablePrefix::AuthRequired)
}
