use nostr_sdk::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub(crate) struct FailingWipeDatabase(MemoryDatabase);

impl FailingWipeDatabase {
    pub(crate) fn new() -> Self {
        Self(MemoryDatabase::with_opts(MemoryDatabaseOptions {
            events: true,
            max_events: Some(64),
        }))
    }
}

#[async_trait]
impl NostrDatabase for FailingWipeDatabase {
    fn backend(&self) -> Backend {
        Backend::Custom("failing-wipe-fixture".to_owned())
    }

    async fn wipe(&self) -> Result<(), DatabaseError> {
        Err(DatabaseError::NotSupported)
    }
}

#[async_trait]
impl NostrEventsDatabase for FailingWipeDatabase {
    async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, DatabaseError> {
        self.0.save_event(event).await
    }

    async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError> {
        self.0.check_id(event_id).await
    }

    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: &Timestamp,
    ) -> Result<bool, DatabaseError> {
        self.0
            .has_coordinate_been_deleted(coordinate, timestamp)
            .await
    }

    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: RelayUrl,
    ) -> Result<(), DatabaseError> {
        self.0.event_id_seen(event_id, relay_url).await
    }

    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<RelayUrl>>, DatabaseError> {
        self.0.event_seen_on_relays(event_id).await
    }

    async fn event_by_id(&self, event_id: &EventId) -> Result<Option<Event>, DatabaseError> {
        self.0.event_by_id(event_id).await
    }

    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
        self.0.count(filters).await
    }

    async fn query(&self, filters: Vec<Filter>) -> Result<Events, DatabaseError> {
        self.0.query(filters).await
    }

    async fn delete(&self, filter: Filter) -> Result<(), DatabaseError> {
        self.0.delete(filter).await
    }
}
