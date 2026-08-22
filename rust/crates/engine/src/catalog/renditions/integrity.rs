use crate::catalog::Catalog;
use crate::PostId;

impl Catalog {
    pub(in crate::catalog) fn apply_known_quarantine(&mut self, post: &PostId) {
        let Some(entry) = self.entries.get_mut(post) else {
            return;
        };
        let Some(digest) = entry.meta.sha256.clone() else {
            return;
        };
        if self
            .quarantined_digests
            .contains(&digest.to_ascii_lowercase())
        {
            entry.quarantine_integrity(&digest, "prior mismatch", 0);
        }
    }

    pub(super) fn update_digest_claim(
        &mut self,
        post: &PostId,
        previous: Option<&str>,
        next: Option<&str>,
    ) {
        let mut changed = false;
        if let Some(previous) = previous.map(str::to_ascii_lowercase) {
            if let Some(posts) = self.digest_claims.get_mut(&previous) {
                changed |= posts.remove(post);
            }
        }
        if let Some(next) = next.map(str::to_ascii_lowercase) {
            changed |= self
                .digest_claims
                .entry(next)
                .or_default()
                .insert(post.clone());
        }
        if changed {
            self.reliability_revision = self.reliability_revision.saturating_add(1);
        }
    }
}
