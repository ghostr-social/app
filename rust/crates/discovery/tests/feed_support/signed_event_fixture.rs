//! Shared typed input for signed Nostr test events.

use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

pub struct SignedEventFixture<'a> {
    pub keys: &'a Keys,
    pub kind: Kind,
    pub content: &'a str,
    pub tags: Vec<Vec<String>>,
    pub created_at: u64,
}

pub fn signed_event(fixture: SignedEventFixture<'_>) -> Event {
    let tags = fixture
        .tags
        .into_iter()
        .map(|parts| Tag::parse(parts).expect("fixture tag"));
    EventBuilder::new(fixture.kind, fixture.content)
        .tags(tags)
        .custom_created_at(Timestamp::from(fixture.created_at))
        .sign_with_keys(fixture.keys)
        .expect("signed fixture event")
}
