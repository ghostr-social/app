use crate::catalog::Catalog;
use crate::PostId;

impl Catalog {
    pub(in crate::catalog) fn apply_known_quarantine(&mut self, post: &PostId) {
        let rejected = self
            .entries
            .get(post)
            .is_some_and(|entry| self.quarantined(post, &entry.meta));
        if let Some(entry) = self.entries.get_mut(post) {
            entry.quarantined = rejected;
        }
    }
}
