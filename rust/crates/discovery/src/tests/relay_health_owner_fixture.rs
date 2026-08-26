use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::relay::registration::{
    RelayRegistration, RelayRegistrationFuture, RelayRegistrationPolicy,
};
use crate::relay::removal::RelayRoleIo;
use crate::tests::relay_io_health_fixture::HealthRelayIo;
use nostr_sdk::Client;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub(crate) struct RegistrationLog(Mutex<Vec<String>>);

impl RelayRegistration for RegistrationLog {
    fn register<'a>(
        &'a self,
        url: &'a str,
        _policy: RelayRegistrationPolicy,
    ) -> RelayRegistrationFuture<'a> {
        self.0.lock().expect("registrations").push(url.to_owned());
        Box::pin(async { Ok(()) })
    }

    fn forget(&self, _url: &str) {}
}

impl RegistrationLog {
    pub(super) fn count(&self, url: &str) -> usize {
        self.0
            .lock()
            .expect("registrations")
            .iter()
            .filter(|item| *item == url)
            .count()
    }
}

pub(crate) fn health_owner(io: Arc<HealthRelayIo>, log: Arc<RegistrationLog>) -> RelayPoolOwner {
    let client = Arc::new(Client::default());
    let roles = RelayRoleIo::with_registration(client, log);
    RelayPoolOwner::with_role_io(RelayPoolConfiguration::default(), io, roles)
}
