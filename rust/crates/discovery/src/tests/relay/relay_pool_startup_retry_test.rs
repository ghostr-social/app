//! Configured relays receive the eager, responsive registration policy.

use crate::relay::registration::{
    RelayRegistration, RelayRegistrationFuture, RelayRegistrationPolicy,
};
use crate::relay::removal::RelayRoleIo;
use crate::relay::roles::{RelayPoolConfiguration, RelayPoolRoles};
use core::time::Duration;
use nostr_sdk::{Client, RelayServiceFlags};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

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
async fn configured_relay_replacement_and_reset_keep_one_bounded_eager_union() {
    let registration = Arc::new(RecordingRegistration::default());
    let io = RelayRoleIo::with_registration(
        Arc::new(Client::default()),
        std::sync::Arc::<RecordingRegistration>::clone(&registration),
    );
    let roles = RelayPoolRoles::new(io, RelayPoolConfiguration::default());
    roles.replace_configuration(configuration()).await;
    let first = take_calls(&registration);
    assert_bounded_policy(&first);

    roles.reset_session().await;
    let reset = take_calls(&registration);
    assert_bounded_policy(&reset);
    assert_eq!(urls(&first), urls(&reset));
}

fn configuration() -> RelayPoolConfiguration {
    let mut search_relays = relay_urls("read", 4);
    search_relays.extend(relay_urls("search", 16));
    RelayPoolConfiguration {
        read_relays: relay_urls("read", 20),
        search_relays,
    }
}

fn take_calls(registration: &RecordingRegistration) -> Vec<RecordedRegistration> {
    core::mem::take(&mut *registration.calls.lock().expect("registration calls"))
}

fn assert_bounded_policy(calls: &[RecordedRegistration]) {
    assert_eq!(calls.len(), 32);
    assert!(urls(calls).contains("wss://search-11.example"));
    assert!(!urls(calls).contains("wss://search-12.example"));
    for call in calls {
        assert_eq!(call.policy.retry_interval, Duration::from_secs(4));
        assert!(call.policy.eager_connect);
        assert_eq!(
            call.policy.flags,
            RelayServiceFlags::PING | RelayServiceFlags::READ
        );
    }
}

fn urls(calls: &[RecordedRegistration]) -> BTreeSet<String> {
    calls.iter().map(|call| call.url.clone()).collect()
}

fn relay_urls(kind: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("wss://{kind}-{index}.example"))
        .collect()
}
