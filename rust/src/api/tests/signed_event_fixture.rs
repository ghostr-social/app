//! Shared typed input for signed Nostr test events.

use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

pub(crate) struct SignedEventFixture<'a> {
    pub(super) keys: &'a Keys,
    pub(super) kind: Kind,
    pub(super) content: &'a str,
    pub(super) tags: Vec<Vec<String>>,
    pub(super) created_at: u64,
}

pub(crate) fn signed_event(fixture: SignedEventFixture<'_>) -> Event {
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
