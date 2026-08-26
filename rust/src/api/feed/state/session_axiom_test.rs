use super::*;

impl FeedState {
    pub(crate) fn ingest_social(&mut self, events: &[Event]) -> Option<Vec<PublicKey>> {
        self.ingest_social_for(self.session, events)
    }
}
