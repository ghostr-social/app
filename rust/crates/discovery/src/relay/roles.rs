//! Reference-counted READ/WRITE roles for the single shared relay pool.

use crate::relay::registration::{RelayRegistration, RelayRegistrationPolicy};
use crate::relay::removal::{RelayRemoval, RelayRoleIo};
use crate::relay::role_book::{unique, DesiredRoles, RoleBook};
use log::warn;
use nostr_sdk::{Client, RelayServiceFlags};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) const MAX_RELAY_READ_FANOUT: usize = 32;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RelayPoolConfiguration {
    pub read_relays: Vec<String>,
    pub search_relays: Vec<String>,
}

#[derive(Clone, Copy)]
pub(crate) enum RelayRole {
    Read,
    Write,
}

pub(crate) struct RelayPoolRoles {
    client: Arc<Client>,
    removal: Arc<dyn RelayRemoval>,
    registration: Arc<dyn RelayRegistration>,
    book: Mutex<RoleBook>,
}

impl RelayPoolRoles {
    pub(crate) fn new(io: RelayRoleIo, configuration: RelayPoolConfiguration) -> Self {
        Self {
            client: io.client,
            removal: io.removal,
            registration: io.registration,
            book: Mutex::new(RoleBook::new(configuration.bounded())),
        }
    }

    pub(super) async fn fallback_read_relays(&self) -> Vec<String> {
        self.book.lock().await.configuration.read_relays.clone()
    }

    pub(crate) async fn acquire(&self, urls: &[String], role: RelayRole) -> Vec<String> {
        let mut book = self.book.lock().await;
        let urls = unique(urls);
        for url in &urls {
            book.acquire(url, role);
            self.sync(url, book.desired(url)).await;
        }
        urls
    }

    pub(crate) async fn release(&self, urls: &[String], role: RelayRole) {
        let mut book = self.book.lock().await;
        for url in urls {
            book.release(url, role);
            self.sync(url, book.desired(url)).await;
        }
    }

    pub(crate) async fn replace_configuration(&self, configuration: RelayPoolConfiguration) {
        let mut book = self.book.lock().await;
        book.clear();
        book.configuration = configuration.bounded();
        self.reconcile(&book).await;
    }

    pub(crate) async fn reset_session(&self) {
        let mut book = self.book.lock().await;
        book.clear();
        self.reconcile(&book).await;
    }

    async fn reconcile(&self, book: &RoleBook) {
        let persistent = book.persistent_relays();
        let current = self.client.relays().await;
        for url in current.keys().map(ToString::to_string) {
            if !persistent.contains(&url) {
                self.remove(&url).await;
            }
        }
        for url in persistent {
            self.sync(&url, book.desired(&url)).await;
        }
    }

    async fn sync(&self, url: &str, desired: DesiredRoles) {
        if !desired.read && !desired.write {
            self.remove(url).await;
            return;
        }
        let policy = RelayRegistrationPolicy::eager(desired.flags());
        if let Err(error) = self.registration.register(url, policy).await {
            warn!("Nostr relay {url} was rejected: {error}");
        }
        self.remove_extra_roles(url, desired).await;
    }

    async fn remove_extra_roles(&self, url: &str, desired: DesiredRoles) {
        let Ok(relay) = self.client.relay(url).await else {
            return;
        };
        let flags = relay.flags();
        if !desired.read && flags.has_read() {
            flags.remove(RelayServiceFlags::READ);
        }
        if !desired.write && flags.has_write() {
            flags.remove(RelayServiceFlags::WRITE);
        }
    }

    async fn remove(&self, url: &str) {
        if self.client.relay(url).await.is_err() {
            return;
        }
        if let Err(error) = self.removal.remove(url).await {
            warn!("Nostr relay {url} could not be removed: {error}");
        } else {
            self.registration.forget(url);
        }
    }
}

impl RelayPoolConfiguration {
    fn bounded(self) -> Self {
        let read_relays = bounded_relay_targets(&self.read_relays);
        let remaining = MAX_RELAY_READ_FANOUT.saturating_sub(read_relays.len());
        let search_relays = unique(&self.search_relays)
            .into_iter()
            .filter(|relay| !read_relays.contains(relay))
            .take(remaining)
            .collect();
        Self {
            read_relays,
            search_relays,
        }
    }
}

pub(crate) fn bounded_relay_targets(relays: &[String]) -> Vec<String> {
    unique(relays)
        .into_iter()
        .take(MAX_RELAY_READ_FANOUT)
        .collect()
}
