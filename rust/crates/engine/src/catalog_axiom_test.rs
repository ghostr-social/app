use super::*;

impl Catalog {
    pub(in super::super) fn len(&self) -> usize {
        self.entries.len()
    }
    pub(in super::super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl CatalogEntry {
    pub(crate) fn evidence(&self) -> &crate::evidence::EvidenceLedger {
        &self.ledger
    }
}
