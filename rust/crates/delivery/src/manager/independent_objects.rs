use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Default)]
pub(crate) struct IndependentObjects {
    required: BTreeSet<TransferIdentity>,
}

impl IndependentObjects {
    pub(super) fn record(&mut self, identity: TransferIdentity) {
        self.required.insert(identity);
    }

    pub(super) fn current(&mut self, catalog: &Catalog) -> HashMap<PostId, HashSet<String>> {
        self.required.retain(|identity| {
            catalog
                .transfer_identity(identity.post(), identity.source().as_str())
                .as_ref()
                == Some(identity)
        });
        let mut current = HashMap::<PostId, HashSet<String>>::new();
        for identity in &self.required {
            current
                .entry(identity.post().clone())
                .or_default()
                .insert(identity.source().as_str().to_owned());
        }
        current
    }

    pub(super) fn clear(&mut self) {
        self.required.clear();
    }
}
