//! Reference-counted READ/WRITE roles for the single shared relay pool.

use crate::discovery::relay_registration::{RelayRegistration, RelayRegistrationPolicy};
use crate::discovery::relay_removal::{RelayRemoval, RelayRoleIo};
use crate::discovery::relay_role_book::{unique, DesiredRoles, RoleBook};
use log::warn;
use nostr_sdk::{Client, RelayServiceFlags};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RelayPoolConfiguration {
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
            book: Mutex::new(RoleBook::new(configuration)),
        }
    }

    pub(crate) async fn fallback_read_relays(&self) -> Vec<String> {
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
        book.configuration = configuration;
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
