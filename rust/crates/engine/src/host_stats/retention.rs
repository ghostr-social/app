use super::{HostRecord, HostStats};

const HOST_CAPACITY: usize = 128;

impl HostStats {
    pub(super) fn record(&mut self, host: &str) -> &mut HostRecord {
        self.make_room_for(host);
        self.hosts.entry(host.to_owned()).or_default()
    }

    pub(super) fn normalize_loaded(&mut self) {
        self.overall.normalize_loaded();
        for record in self.hosts.values_mut() {
            record.normalize_loaded();
        }
        while self.hosts.len() > HOST_CAPACITY {
            let victim = self.oldest_host().expect("nonempty over-capacity hosts");
            self.hosts.remove(&victim);
        }
    }

    fn make_room_for(&mut self, host: &str) {
        if self.hosts.contains_key(host) || self.hosts.len() < HOST_CAPACITY {
            return;
        }
        if let Some(victim) = self.oldest_host() {
            self.hosts.remove(&victim);
        }
    }

    fn oldest_host(&self) -> Option<String> {
        self.hosts
            .iter()
            .min_by_key(|(host, record)| (record.last_observed_at_ms(), *host))
            .map(|(host, _)| host.clone())
    }
}
