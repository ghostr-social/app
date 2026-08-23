use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct IndependentObjects {
    required: HashSet<TransferIdentity>,
}

impl IndependentObjects {
    pub(crate) fn record(&mut self, identity: TransferIdentity) {
        self.required.insert(identity);
    }

    pub(crate) fn current(&mut self, catalog: &Catalog) -> HashMap<PostId, HashSet<String>> {
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

    pub(crate) fn clear(&mut self) {
        self.required.clear();
    }
}
