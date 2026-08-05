//! Configured relays receive the eager, responsive registration policy.

use crate::discovery::relay_pool_roles::{RelayPoolConfiguration, RelayPoolRoles};
use crate::discovery::relay_registration::{
    RelayRegistration, RelayRegistrationFuture, RelayRegistrationPolicy,
};
use crate::discovery::relay_removal::RelayRoleIo;
use nostr_sdk::{Client, RelayServiceFlags};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone)]
struct RecordedRegistration {
    url: String,
    policy: RelayRegistrationPolicy,
}

#[derive(Default)]
struct RecordingRegistration {
    calls: Mutex<Vec<RecordedRegistration>>,
}

impl RelayRegistration for RecordingRegistration {
    fn register<'a>(
        &'a self,
        url: &'a str,
        policy: RelayRegistrationPolicy,
    ) -> RelayRegistrationFuture<'a> {
        self.calls
            .lock()
            .expect("registration calls")
            .push(RecordedRegistration {
                url: url.to_owned(),
                policy,
            });
        Box::pin(async { Ok(()) })
    }

    fn forget(&self, _url: &str) {}
}

#[tokio::test]
async fn configured_relays_connect_eagerly_with_a_four_second_base() {
    let registration = Arc::new(RecordingRegistration::default());
    let io = RelayRoleIo::with_registration(Arc::new(Client::default()), registration.clone());
    let roles = RelayPoolRoles::new(
        io,
        RelayPoolConfiguration {
            read_relays: vec!["wss://read.example".to_owned()],
            search_relays: vec!["wss://search.example".to_owned()],
        },
    );

    roles.reset_session().await;

    let calls = registration.calls.lock().expect("registration calls");
    assert_eq!(calls.len(), 2);
    for call in calls.iter() {
        assert!(call.url.ends_with(".example"));
        assert_eq!(call.policy.retry_interval, Duration::from_secs(4));
        assert!(call.policy.eager_connect);
        assert_eq!(
            call.policy.flags,
            RelayServiceFlags::PING | RelayServiceFlags::READ
        );
    }
}
