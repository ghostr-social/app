//! Shared typed input for signed Nostr test events.

use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

pub(in crate::tests) struct SignedEventFixture<'a> {
    pub(in crate::tests) keys: &'a Keys,
    pub(in crate::tests) kind: Kind,
    pub(in crate::tests) content: &'a str,
    pub(in crate::tests) tags: Vec<Vec<String>>,
    pub(in crate::tests) created_at: u64,
}

pub(in crate::tests) fn signed_event(fixture: SignedEventFixture<'_>) -> Event {
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
