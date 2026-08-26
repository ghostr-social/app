use super::*;

impl OutboxDirectory {
    /// Ingests a whole retrieval's events; anything that is not a
    /// kind-10002 relay list is ignored.
    pub(crate) fn ingest_all(&mut self, events: &[Event]) {
        for event in events {
            self.ingest(event);
        }
    }
}
