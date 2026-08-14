//! Lookup of anchored and bounded pending deletion claims.

use super::deletions::{DeletionClaim, DeletionTarget};
use super::parsing::ParsedVideoPost;
use super::pending_deletions::PendingDeletions;
use super::reposts::RepostTarget;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct DeletionKey {
    target: DeletionTarget,
    deleter_pubkey: String,
}

#[derive(Debug, Default)]
pub struct DeletionIndex {
    anchored: HashMap<DeletionKey, Option<u64>>,
    pending: PendingDeletions,
}

impl DeletionIndex {
    pub fn ingest(&mut self, claims: Vec<DeletionClaim>) -> bool {
        claims
            .into_iter()
            .fold(false, |changed, claim| self.insert(claim) | changed)
    }

    pub fn reanchor(&mut self, posts: &[ParsedVideoPost]) {
        let mut previous = std::mem::take(&mut self.anchored);
        let mut anchored = HashMap::new();
        for post in posts {
            for key in keys_for_post(post) {
                self.anchor(key, &mut previous, &mut anchored);
            }
        }
        self.demote(previous);
        self.anchored = anchored;
    }

    pub fn deletes_content(&self, post: &ParsedVideoPost) -> bool {
        if self.event_deleted(&post.event_id, &post.author_pubkey) {
            return true;
        }
        address_key(post).is_some_and(|key| {
            self.deleted_at(&key)
                .is_some_and(|deleted_at| post.created_at <= deleted_at)
        })
    }

    pub fn deletes_occurrence(&self, post: &ParsedVideoPost) -> bool {
        let Some(repost) = &post.repost else {
            return self.deletes_content(post);
        };
        if self.event_deleted(&repost.event_id, &repost.reposter_pubkey) {
            return true;
        }
        repost.target == RepostTarget::SpecificEvent && self.deletes_content(post)
    }

    fn insert(&mut self, claim: DeletionClaim) -> bool {
        let key = DeletionKey::new(claim.target, claim.deleter_pubkey);
        match self.anchored.get_mut(&key) {
            Some(current) => replace_if_newer(current, claim.deleted_at),
            None => {
                self.pending.insert(key, claim.deleted_at);
                false
            }
        }
    }

    fn anchor(
        &mut self,
        key: DeletionKey,
        previous: &mut HashMap<DeletionKey, Option<u64>>,
        anchored: &mut HashMap<DeletionKey, Option<u64>>,
    ) {
        if anchored.contains_key(&key) {
            return;
        }
        let claim = previous
            .remove(&key)
            .flatten()
            .or_else(|| self.pending.take(&key));
        anchored.insert(key, claim);
    }

    fn demote(&mut self, previous: HashMap<DeletionKey, Option<u64>>) {
        for (key, claim) in previous {
            if let Some(deleted_at) = claim {
                self.pending.insert(key, deleted_at);
            }
        }
    }

    fn event_deleted(&self, event_id: &str, author: &str) -> bool {
        self.deleted_at(&DeletionKey::event(event_id, author))
            .is_some()
    }

    fn deleted_at(&self, key: &DeletionKey) -> Option<u64> {
        self.anchored
            .get(key)
            .copied()
            .flatten()
            .or_else(|| self.pending.get(key))
    }

    #[cfg(test)]
    pub(crate) fn with_retention(retention: usize) -> Self {
        Self {
            pending: PendingDeletions::with_retention(retention),
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn retained_claims(&self) -> usize {
        self.anchored
            .values()
            .filter(|claim| claim.is_some())
            .count()
            + self.pending.len()
    }
}

impl DeletionKey {
    fn new(target: DeletionTarget, deleter_pubkey: String) -> Self {
        Self {
            target,
            deleter_pubkey,
        }
    }

    fn event(event_id: &str, author: &str) -> Self {
        Self::new(
            DeletionTarget::Event(event_id.to_owned()),
            author.to_owned(),
        )
    }
}

fn keys_for_post(post: &ParsedVideoPost) -> Vec<DeletionKey> {
    let mut keys = vec![DeletionKey::event(&post.event_id, &post.author_pubkey)];
    keys.extend(address_key(post));
    if let Some(repost) = &post.repost {
        keys.push(DeletionKey::event(
            &repost.event_id,
            &repost.reposter_pubkey,
        ));
    }
    keys
}

fn address_key(post: &ParsedVideoPost) -> Option<DeletionKey> {
    post.published_identifier.as_ref()?;
    Some(DeletionKey::new(
        DeletionTarget::Address(post.coordinate()),
        post.author_pubkey.clone(),
    ))
}

fn replace_if_newer(current: &mut Option<u64>, incoming: u64) -> bool {
    if current.is_some_and(|stored| stored >= incoming) {
        return false;
    }
    *current = Some(incoming);
    true
}
