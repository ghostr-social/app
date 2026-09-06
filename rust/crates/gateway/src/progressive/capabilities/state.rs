use super::{ProgressiveAssetAuthority, ProgressiveCapabilityId, TOKEN_BYTES};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use core::time::Duration;
use rand::RngCore as _;
use std::collections::HashMap;
use tokio::time::Instant;

#[derive(Default)]
pub(super) struct CapabilityState {
    entries: HashMap<ProgressiveCapabilityId, CapabilityLease>,
}

impl CapabilityState {
    pub(super) fn existing(
        &self,
        authority: &ProgressiveAssetAuthority,
    ) -> Option<ProgressiveCapabilityId> {
        self.entries
            .iter()
            .find_map(|(id, lease)| (lease.authority == *authority).then(|| id.clone()))
    }

    pub(super) fn prune(&mut self, now: Instant, ttl: Duration) {
        self.entries
            .retain(|_, lease| now.duration_since(lease.last_used) < ttl);
    }

    pub(super) fn refresh(
        &mut self,
        authority: &ProgressiveAssetAuthority,
        now: Instant,
    ) -> Option<ProgressiveCapabilityId> {
        let id = self.existing(authority)?;
        self.entries.get_mut(&id)?.last_used = now;
        Some(id)
    }

    pub(super) fn make_room(&mut self, capacity: usize) {
        if self.entries.len() < capacity {
            return;
        }
        let oldest = self
            .entries
            .iter()
            .min_by_key(|(_, lease)| lease.last_used)
            .map(|(id, _)| id.clone());
        if let Some(id) = oldest {
            self.entries.remove(&id);
        }
    }

    pub(super) fn insert(
        &mut self,
        authority: ProgressiveAssetAuthority,
        now: Instant,
    ) -> ProgressiveCapabilityId {
        let id = self.unique_id();
        self.entries.insert(
            id.clone(),
            CapabilityLease {
                authority,
                last_used: now,
            },
        );
        id
    }

    pub(super) fn recognizes(&self, id: &ProgressiveCapabilityId, post: &str) -> bool {
        self.entries
            .get(id)
            .is_some_and(|lease| lease.authority.post() == post)
    }

    pub(super) fn authorize(
        &mut self,
        id: &ProgressiveCapabilityId,
        post: &str,
        authority: &ProgressiveAssetAuthority,
        now: Instant,
    ) -> bool {
        let Some(lease) = self.entries.get_mut(id) else {
            return false;
        };
        if lease.authority.post() != post || lease.authority != *authority {
            return false;
        }
        lease.last_used = now;
        true
    }

    fn unique_id(&self) -> ProgressiveCapabilityId {
        loop {
            let candidate = random_id();
            if !self.entries.contains_key(&candidate) {
                return candidate;
            }
        }
    }
}

struct CapabilityLease {
    authority: ProgressiveAssetAuthority,
    last_used: Instant,
}

fn random_id() -> ProgressiveCapabilityId {
    let mut bytes = [0_u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    ProgressiveCapabilityId(URL_SAFE_NO_PAD.encode(bytes))
}
