use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::relay::registration::{
    RelayRegistration, RelayRegistrationFuture, RelayRegistrationPolicy,
};
use crate::relay::removal::RelayRoleIo;
use crate::test_support::{read_request, TestRelayIo};
use nostr_sdk::Client;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct RegistrationSpy {
    calls: Mutex<Vec<String>>,
}

impl RelayRegistration for RegistrationSpy {
    fn register<'a>(
        &'a self,
        url: &'a str,
        _policy: RelayRegistrationPolicy,
    ) -> RelayRegistrationFuture<'a> {
        self.calls
            .lock()
            .expect("valid test fixture")
            .push(url.to_owned());
        Box::pin(async { Ok(()) })
    }

    fn forget(&self, _url: &str) {}
}

#[tokio::test]
async fn owner_rejects_excessive_fanout_before_registration_or_read() {
    let client = Arc::new(Client::default());
    let registration = Arc::new(RegistrationSpy::default());
    let roles = RelayRoleIo::with_registration(
        std::sync::Arc::clone(&client),
        std::sync::Arc::<RegistrationSpy>::clone(&registration),
    );
    let io = Arc::new(TestRelayIo::blocked());
    io.release_query();
    let owner = RelayPoolOwner::with_role_io(
        RelayPoolConfiguration::default(),
        std::sync::Arc::<TestRelayIo>::clone(&io),
        roles,
    );
    let mut request = read_request("wss://unused.example");
    request.relays = Some(
        (0..33)
            .map(|index| format!("wss://relay-{index}.example"))
            .collect(),
    );

    let error = owner.read(request).await.expect_err("fanout must fail");

    assert!(error.message.contains("relay fanout exceeds 32"));
    assert!(registration
        .calls
        .lock()
        .expect("valid test fixture")
        .is_empty());
    assert_eq!(io.read_count(), 0);
}

#[tokio::test]
async fn configured_fallback_is_stably_capped_before_registration() {
    let client = Arc::new(Client::default());
    let registration = Arc::new(RegistrationSpy::default());
    let roles = RelayRoleIo::with_registration(
        client,
        std::sync::Arc::<RegistrationSpy>::clone(&registration),
    );
    let io = Arc::new(TestRelayIo::blocked());
    io.release_query();
    let configured: Vec<_> = (0..33)
        .map(|index| format!("wss://relay-{index}.example"))
        .collect();
    let owner = RelayPoolOwner::with_role_io(
        RelayPoolConfiguration {
            read_relays: configured.clone(),
            search_relays: Vec::new(),
        },
        std::sync::Arc::<TestRelayIo>::clone(&io),
        roles,
    );
    let mut request = read_request("wss://unused.example");
    request.relays = None;

    owner.read(request).await.expect("bounded fallback read");

    let calls: BTreeSet<_> = registration
        .calls
        .lock()
        .expect("valid test fixture")
        .iter()
        .cloned()
        .collect();
    assert_eq!(calls.len(), 32);
    assert!(!calls.contains(&configured[32]));
    assert_eq!(io.read_count(), 1);
}
