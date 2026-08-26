use super::*;

impl OutboxBootstrap {
    /// Adopts a landed follow set as the main feed's routing set and
    /// chases the relay lists it does not know yet.
    pub(crate) async fn track_follows(&self, follows: Vec<PublicKey>) {
        let generation = locked(&self.session).generation;
        self.track_follows_for(generation, follows).await;
    }
    /// Files every relay list in a retrieval's events. Every page flows
    /// through here, so a page carrying none never takes the lock.
    pub(crate) async fn ingest(&self, events: &[Event]) {
        let generation = locked(&self.session).generation;
        self.ingest_for(generation, events).await;
    }
}
