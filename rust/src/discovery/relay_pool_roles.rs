//! Reference-counted READ/WRITE roles for the single shared relay pool.

use crate::discovery::relay_removal::{RelayRemoval, RelayRoleIo};
use log::warn;
use nostr_sdk::{Client, RelayServiceFlags};
use std::collections::{HashMap, HashSet};
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

#[derive(Clone, Copy, Default)]
struct RelayUse {
    reads: usize,
    writes: usize,
}

#[derive(Clone, Copy)]
struct DesiredRoles {
    read: bool,
    write: bool,
}

struct RoleBook {
    configuration: RelayPoolConfiguration,
    active: HashMap<String, RelayUse>,
}

pub(crate) struct RelayPoolRoles {
    client: Arc<Client>,
    removal: Arc<dyn RelayRemoval>,
    book: Mutex<RoleBook>,
}

impl RelayPoolRoles {
    pub(crate) fn new(io: RelayRoleIo, configuration: RelayPoolConfiguration) -> Self {
        Self {
            client: io.client,
            removal: io.removal,
            book: Mutex::new(RoleBook {
                configuration,
                active: HashMap::new(),
            }),
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
        book.active.clear();
        book.configuration = configuration;
        self.reconcile(&book).await;
    }

    pub(crate) async fn reset_session(&self) {
        let mut book = self.book.lock().await;
        book.active.clear();
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
        if desired.read {
            self.add_read(url).await;
        }
        if desired.write {
            self.add_write(url).await;
        }
        self.remove_extra_roles(url, desired).await;
    }

    async fn add_read(&self, url: &str) {
        if let Err(error) = self.client.add_read_relay(url).await {
            warn!("Nostr read relay {url} was rejected: {error}");
        }
    }

    async fn add_write(&self, url: &str) {
        if let Err(error) = self.client.add_write_relay(url).await {
            warn!("Nostr write relay {url} was rejected: {error}");
        }
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
        }
    }
}

impl RoleBook {
    fn acquire(&mut self, url: &str, role: RelayRole) {
        let usage = self.active.entry(url.to_owned()).or_default();
        match role {
            RelayRole::Read => usage.reads += 1,
            RelayRole::Write => usage.writes += 1,
        }
    }

    fn release(&mut self, url: &str, role: RelayRole) {
        let Some(usage) = self.active.get_mut(url) else {
            return;
        };
        match role {
            RelayRole::Read => usage.reads = usage.reads.saturating_sub(1),
            RelayRole::Write => usage.writes = usage.writes.saturating_sub(1),
        }
        if usage.reads == 0 && usage.writes == 0 {
            self.active.remove(url);
        }
    }

    fn desired(&self, url: &str) -> DesiredRoles {
        let usage = self.active.get(url).copied().unwrap_or_default();
        DesiredRoles {
            read: self.persistent_relays().contains(url) || usage.reads > 0,
            write: usage.writes > 0,
        }
    }

    fn persistent_relays(&self) -> HashSet<String> {
        self.configuration
            .read_relays
            .iter()
            .chain(&self.configuration.search_relays)
            .cloned()
            .collect()
    }
}

fn unique(urls: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    urls.iter()
        .filter(|url| seen.insert(url.as_str()))
        .cloned()
        .collect()
}
