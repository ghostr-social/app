use crate::relay::roles::{RelayPoolConfiguration, RelayRole};
use nostr_sdk::RelayServiceFlags;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Default)]
struct RelayUse {
    reads: usize,
    writes: usize,
}

#[derive(Clone, Copy)]
pub(super) struct DesiredRoles {
    pub(super) read: bool,
    pub(super) write: bool,
}

pub(super) struct RoleBook {
    pub(super) configuration: RelayPoolConfiguration,
    active: HashMap<String, RelayUse>,
}

impl RoleBook {
    pub(super) fn new(configuration: RelayPoolConfiguration) -> Self {
        Self {
            configuration,
            active: HashMap::new(),
        }
    }

    pub(super) fn clear(&mut self) {
        self.active.clear();
    }

    pub(super) fn acquire(&mut self, url: &str, role: RelayRole) {
        let usage = self.active.entry(url.to_owned()).or_default();
        match role {
            RelayRole::Read => usage.reads += 1,
            RelayRole::Write => usage.writes += 1,
        }
    }

    pub(super) fn release(&mut self, url: &str, role: RelayRole) {
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

    pub(super) fn desired(&self, url: &str) -> DesiredRoles {
        let usage = self.active.get(url).copied().unwrap_or_default();
        DesiredRoles {
            read: self.persistent_relays().contains(url) || usage.reads > 0,
            write: usage.writes > 0,
        }
    }

    pub(super) fn persistent_relays(&self) -> HashSet<String> {
        self.configuration
            .read_relays
            .iter()
            .chain(&self.configuration.search_relays)
            .cloned()
            .collect()
    }
}

impl DesiredRoles {
    pub(super) fn flags(self) -> RelayServiceFlags {
        let mut flags = RelayServiceFlags::PING;
        if self.read {
            flags |= RelayServiceFlags::READ;
        }
        if self.write {
            flags |= RelayServiceFlags::WRITE;
        }
        flags
    }
}

pub(super) fn unique(urls: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    urls.iter()
        .filter(|url| seen.insert(url.as_str()))
        .cloned()
        .collect()
}
